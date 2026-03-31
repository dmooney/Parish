//! HTTP route handlers for the Parish web server.
//!
//! Each route extracts a session ID from the query string and operates
//! on that session's isolated game state. The inference pipeline is shared.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use tokio::sync::mpsc;

use parish_core::config::Provider;
use parish_core::inference::openai_client::OpenAiClient;
use parish_core::inference::{InferenceQueue, new_inference_log, spawn_inference_worker};
use parish_core::input::{InputResult, classify_input, parse_intent_local};
use parish_core::ipc::{
    LoadingPayload, MapData, NpcInfo, StreamEndPayload, TextLogPayload, ThemePalette,
    WorldSnapshot, capitalize_first,
};
use parish_core::npc::parse_npc_stream_response;
use parish_core::npc::ticks;
use parish_core::world::description::{format_exits, render_description};
use parish_core::world::movement::{self, MovementResult};

use parish_core::debug_snapshot::{self, DebugSnapshot, InferenceDebug};

use crate::state::{GameConfig, GameSession, ServerState};

/// Monotonically increasing request ID counter for inference requests.
static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// Query parameter for session identification.
#[derive(serde::Deserialize)]
pub struct SessionQuery {
    /// The session UUID.
    pub session: String,
}

/// Response body for session creation.
#[derive(serde::Serialize)]
pub struct CreateSessionResponse {
    /// The newly created session UUID.
    pub session_id: String,
}

/// Helper: look up a session and touch its activity timestamp.
async fn get_session(
    state: &Arc<ServerState>,
    session_id: &str,
) -> Result<Arc<GameSession>, StatusCode> {
    let session = state
        .sessions
        .get(session_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    session.touch().await;
    Ok(session)
}

// ── Session management ─────────────────────────────────────────────────────

/// `POST /api/session` — creates a new game session.
pub async fn create_session(
    State(state): State<Arc<ServerState>>,
) -> Result<Json<CreateSessionResponse>, StatusCode> {
    let (id, session) = state
        .sessions
        .create_session()
        .await
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Spawn per-session background ticks
    crate::spawn_session_ticks(Arc::clone(&session));

    tracing::info!("Created session {}", id);
    Ok(Json(CreateSessionResponse { session_id: id }))
}

// ── Query endpoints ─────────────────────────────────────────────────────────

/// `GET /api/world-snapshot` — returns the current world snapshot.
pub async fn get_world_snapshot(
    State(state): State<Arc<ServerState>>,
    Query(q): Query<SessionQuery>,
) -> Result<Json<WorldSnapshot>, StatusCode> {
    let session = get_session(&state, &q.session).await?;
    let world = session.world.lock().await;
    Ok(Json(parish_core::ipc::snapshot_from_world(&world)))
}

/// `GET /api/map` — returns the map with all locations and edges.
pub async fn get_map(
    State(state): State<Arc<ServerState>>,
    Query(q): Query<SessionQuery>,
) -> Result<Json<MapData>, StatusCode> {
    let session = get_session(&state, &q.session).await?;
    let world = session.world.lock().await;
    Ok(Json(parish_core::ipc::build_map_data(&world)))
}

/// `GET /api/npcs-here` — returns NPCs at the player's current location.
pub async fn get_npcs_here(
    State(state): State<Arc<ServerState>>,
    Query(q): Query<SessionQuery>,
) -> Result<Json<Vec<NpcInfo>>, StatusCode> {
    let session = get_session(&state, &q.session).await?;
    let world = session.world.lock().await;
    let npc_manager = session.npc_manager.lock().await;
    Ok(Json(parish_core::ipc::build_npcs_here(
        &world,
        &npc_manager,
    )))
}

/// `GET /api/theme` — returns the current time-of-day theme palette.
pub async fn get_theme(
    State(state): State<Arc<ServerState>>,
    Query(q): Query<SessionQuery>,
) -> Result<Json<ThemePalette>, StatusCode> {
    let session = get_session(&state, &q.session).await?;
    let world = session.world.lock().await;
    Ok(Json(parish_core::ipc::build_theme(&world)))
}

/// `GET /api/debug-snapshot` — returns full debug state for the debug panel.
pub async fn get_debug_snapshot(
    State(state): State<Arc<ServerState>>,
    Query(q): Query<SessionQuery>,
) -> Result<Json<DebugSnapshot>, StatusCode> {
    let session = get_session(&state, &q.session).await?;
    let world = session.world.lock().await;
    let npc_manager = session.npc_manager.lock().await;
    let config = state.config.lock().await;
    let events = std::collections::VecDeque::new();
    let inference = InferenceDebug {
        provider_name: config.provider_name.clone(),
        model_name: config.model_name.clone(),
        base_url: config.base_url.clone(),
        cloud_provider: config.cloud_provider_name.clone(),
        cloud_model: config.cloud_model_name.clone(),
        has_queue: state.inference_queue.lock().await.is_some(),
        improv_enabled: config.improv_enabled,
        call_log: Vec::new(),
    };
    Ok(Json(debug_snapshot::build_debug_snapshot(
        &world,
        &npc_manager,
        &events,
        &inference,
    )))
}

// ── Input endpoint ──────────────────────────────────────────────────────────

/// Request body for `POST /api/submit-input`.
#[derive(serde::Deserialize)]
pub struct SubmitInputRequest {
    /// The player's input text.
    pub text: String,
}

/// `POST /api/submit-input` — processes player text input.
pub async fn submit_input(
    State(state): State<Arc<ServerState>>,
    Query(q): Query<SessionQuery>,
    Json(body): Json<SubmitInputRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let session = get_session(&state, &q.session).await?;
    let text = body.text.trim().to_string();
    if text.is_empty() {
        return Ok(StatusCode::OK);
    }

    // Emit the player's own text as a log entry
    session.event_bus.emit(
        "text-log",
        &TextLogPayload {
            source: "player".to_string(),
            content: format!("> {}", text),
        },
    );

    match classify_input(&text) {
        InputResult::SystemCommand(cmd) => {
            handle_system_command(cmd, &session, &state).await;
        }
        InputResult::GameInput(raw) => {
            handle_game_input(raw, &session, &state).await;
        }
    }

    Ok(StatusCode::OK)
}

// ── Internal helpers ────────────────────────────────────────────────────────

/// Rebuilds the inference pipeline after a provider/key/client change.
async fn rebuild_inference(state: &Arc<ServerState>) {
    let config = state.config.lock().await;
    let new_client = OpenAiClient::new(&config.base_url, config.api_key.as_deref());
    drop(config);

    let mut client_guard = state.client.lock().await;
    *client_guard = Some(new_client.clone());
    drop(client_guard);

    let (tx, rx) = tokio::sync::mpsc::channel(32);
    let _worker = spawn_inference_worker(new_client, rx, new_inference_log());
    let queue = InferenceQueue::new(tx);
    let mut iq = state.inference_queue.lock().await;
    *iq = Some(queue);
}

/// Handles `/command` system inputs.
async fn handle_system_command(
    cmd: parish_core::input::Command,
    session: &Arc<GameSession>,
    state: &Arc<ServerState>,
) {
    use parish_core::input::Command;
    use parish_core::ipc::mask_key;

    let mut needs_rebuild = false;

    let response = match cmd {
        Command::Pause => {
            let mut world = session.world.lock().await;
            world.clock.pause();
            "The clocks of the parish stand still.".to_string()
        }
        Command::Resume => {
            let mut world = session.world.lock().await;
            world.clock.resume();
            "Time stirs again in the parish.".to_string()
        }
        Command::Status => {
            let world = session.world.lock().await;
            let tod = world.clock.time_of_day();
            let season = world.clock.season();
            let loc = world.current_location().name.clone();
            let paused = if world.clock.is_paused() {
                " (paused)"
            } else {
                ""
            };
            format!("Location: {} | {} | {}{}", loc, tod, season, paused)
        }
        Command::Help => [
            "A few things ye might say:",
            "  /help     — Show this help",
            "  /pause    — Hold time still",
            "  /resume   — Let time flow again",
            "  /speed    — Show or change game speed",
            "  /status   — Where am I?",
        ]
        .join("\n"),
        Command::Quit => {
            "The web server cannot be quit from the game. Close your browser tab.".to_string()
        }
        Command::ShowSpeed => {
            let world = session.world.lock().await;
            let s = world
                .clock
                .current_speed()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("Custom ({}x)", world.clock.speed_factor()));
            format!("Speed: {}", s)
        }
        Command::SetSpeed(speed) => {
            let mut world = session.world.lock().await;
            world.clock.set_speed(speed);
            speed.activation_message().to_string()
        }
        Command::InvalidSpeed(name) => {
            format!(
                "Unknown speed '{}'. Try: slow, normal, fast, fastest.",
                name
            )
        }
        Command::ToggleSidebar => "The Irish words panel is managed by the sidebar.".to_string(),
        Command::ToggleImprov => {
            let mut config = state.config.lock().await;
            config.improv_enabled = !config.improv_enabled;
            if config.improv_enabled {
                "The characters loosen up — improv craft engaged.".to_string()
            } else {
                "The characters settle back to their usual selves.".to_string()
            }
        }
        Command::ShowProvider => {
            let config = state.config.lock().await;
            format!("Provider: {}", config.provider_name)
        }
        Command::SetProvider(name) => match Provider::from_str_loose(&name) {
            Ok(provider) => {
                let mut config = state.config.lock().await;
                config.base_url = provider.default_base_url().to_string();
                config.provider_name = format!("{:?}", provider).to_lowercase();
                needs_rebuild = true;
                format!("Provider changed to {}.", config.provider_name)
            }
            Err(e) => format!("{}", e),
        },
        Command::ShowModel => {
            let config = state.config.lock().await;
            if config.model_name.is_empty() {
                "Model: (auto-detect)".to_string()
            } else {
                format!("Model: {}", config.model_name)
            }
        }
        Command::SetModel(name) => {
            let mut config = state.config.lock().await;
            config.model_name = name.clone();
            format!("Model changed to {}.", name)
        }
        Command::ShowKey => {
            let config = state.config.lock().await;
            match &config.api_key {
                Some(key) => format!("API key: {}", mask_key(key)),
                None => "API key: (not set)".to_string(),
            }
        }
        Command::SetKey(value) => {
            let mut config = state.config.lock().await;
            config.api_key = Some(value);
            needs_rebuild = true;
            "API key updated.".to_string()
        }
        Command::ShowCloud => {
            let config = state.config.lock().await;
            if let Some(ref provider) = config.cloud_provider_name {
                let model = config.cloud_model_name.as_deref().unwrap_or("(none)");
                format!("Cloud: {} | Model: {}", provider, model)
            } else {
                "No cloud provider configured.".to_string()
            }
        }
        Command::SetCloudProvider(name) => match Provider::from_str_loose(&name) {
            Ok(provider) => {
                let mut config = state.config.lock().await;
                let base_url = provider.default_base_url().to_string();
                let provider_name = format!("{:?}", provider).to_lowercase();
                config.cloud_provider_name = Some(provider_name.clone());
                config.cloud_base_url = Some(base_url.clone());
                let mut cloud_guard = state.cloud_client.lock().await;
                *cloud_guard = Some(OpenAiClient::new(
                    &base_url,
                    config.cloud_api_key.as_deref(),
                ));
                format!("Cloud provider changed to {}.", provider_name)
            }
            Err(e) => format!("{}", e),
        },
        Command::ShowCloudModel => {
            let config = state.config.lock().await;
            match &config.cloud_model_name {
                Some(model) => format!("Cloud model: {}", model),
                None => "Cloud model: (not set)".to_string(),
            }
        }
        Command::SetCloudModel(name) => {
            let mut config = state.config.lock().await;
            config.cloud_model_name = Some(name.clone());
            format!("Cloud model changed to {}.", name)
        }
        Command::ShowCloudKey => {
            let config = state.config.lock().await;
            match &config.cloud_api_key {
                Some(key) => format!("Cloud API key: {}", mask_key(key)),
                None => "Cloud API key: (not set)".to_string(),
            }
        }
        Command::SetCloudKey(value) => {
            let mut config = state.config.lock().await;
            config.cloud_api_key = Some(value);
            let base_url = config
                .cloud_base_url
                .as_deref()
                .unwrap_or("https://openrouter.ai/api")
                .to_string();
            let mut cloud_guard = state.cloud_client.lock().await;
            *cloud_guard = Some(OpenAiClient::new(
                &base_url,
                config.cloud_api_key.as_deref(),
            ));
            "Cloud API key updated.".to_string()
        }
        Command::Save | Command::Fork(_) | Command::Load(_) | Command::Branches | Command::Log => {
            "Persistence is not yet available in web mode.".to_string()
        }
        Command::ShowCategoryProvider(cat) => {
            let config = state.config.lock().await;
            let idx = GameConfig::cat_idx(cat);
            match &config.category_provider[idx] {
                Some(p) => format!("{} provider: {}", cat.name(), p),
                None => format!(
                    "{} provider: (inherits base: {})",
                    cat.name(),
                    config.provider_name
                ),
            }
        }
        Command::SetCategoryProvider(cat, name) => match Provider::from_str_loose(&name) {
            Ok(provider) => {
                let mut config = state.config.lock().await;
                let idx = GameConfig::cat_idx(cat);
                let provider_name = format!("{:?}", provider).to_lowercase();
                config.category_provider[idx] = Some(provider_name.clone());
                config.category_base_url[idx] = Some(provider.default_base_url().to_string());
                format!("{} provider changed to {}.", cat.name(), provider_name)
            }
            Err(e) => format!("{}", e),
        },
        Command::ShowCategoryModel(cat) => {
            let config = state.config.lock().await;
            let idx = GameConfig::cat_idx(cat);
            match &config.category_model[idx] {
                Some(m) => format!("{} model: {}", cat.name(), m),
                None => format!(
                    "{} model: (inherits base: {})",
                    cat.name(),
                    config.model_name
                ),
            }
        }
        Command::SetCategoryModel(cat, name) => {
            let mut config = state.config.lock().await;
            let idx = GameConfig::cat_idx(cat);
            config.category_model[idx] = Some(name.clone());
            format!("{} model changed to {}.", cat.name(), name)
        }
        Command::ShowCategoryKey(cat) => {
            let config = state.config.lock().await;
            let idx = GameConfig::cat_idx(cat);
            match &config.category_api_key[idx] {
                Some(key) => format!("{} API key: {}", cat.name(), mask_key(key)),
                None => format!("{} API key: (not set)", cat.name()),
            }
        }
        Command::SetCategoryKey(cat, value) => {
            let cat_name = cat.name().to_string();
            let mut config = state.config.lock().await;
            let idx = GameConfig::cat_idx(cat);
            config.category_api_key[idx] = Some(value);
            format!("{} API key updated.", cat_name)
        }
        Command::Debug(_) => "Debug commands are not available in web mode.".to_string(),
        Command::Spinner(_) => "Spinner customization is not available in web mode.".to_string(),
        Command::About => "Parish — An Irish Living World Text Adventure (web mode).".to_string(),
    };

    if needs_rebuild {
        rebuild_inference(state).await;
    }

    session.event_bus.emit(
        "text-log",
        &TextLogPayload {
            source: "system".to_string(),
            content: response,
        },
    );

    let world = session.world.lock().await;
    session.event_bus.emit(
        "world-update",
        &parish_core::ipc::snapshot_from_world(&world),
    );
}

/// Handles free-form game input: parses intent then dispatches.
async fn handle_game_input(raw: String, session: &Arc<GameSession>, state: &Arc<ServerState>) {
    let intent = parse_intent_local(&raw);

    let is_move = intent
        .as_ref()
        .map(|i| matches!(i.intent, parish_core::input::IntentKind::Move))
        .unwrap_or(false);
    let is_look = intent
        .as_ref()
        .map(|i| matches!(i.intent, parish_core::input::IntentKind::Look))
        .unwrap_or(false);
    let move_target = intent
        .as_ref()
        .filter(|_i| is_move)
        .and_then(|i| i.target.clone());

    if is_move {
        if let Some(target) = move_target {
            handle_movement(&target, session).await;
        } else {
            session.event_bus.emit(
                "text-log",
                &TextLogPayload {
                    source: "system".to_string(),
                    content: "And where would ye be off to?".to_string(),
                },
            );
        }
        return;
    }

    if is_look {
        handle_look(session).await;
        return;
    }

    handle_npc_conversation(raw, session, state).await;
}

/// Resolves movement to a named location.
async fn handle_movement(target: &str, session: &Arc<GameSession>) {
    let result = {
        let world = session.world.lock().await;
        movement::resolve_movement(target, &world.graph, world.player_location)
    };

    match result {
        MovementResult::Arrived {
            destination,
            minutes,
            narration,
            ..
        } => {
            {
                let mut world = session.world.lock().await;
                world.clock.advance(minutes as i64);
                world.player_location = destination;

                let new_loc =
                    world
                        .graph
                        .get(destination)
                        .map(|data| parish_core::world::Location {
                            id: destination,
                            name: data.name.clone(),
                            description: data.description_template.clone(),
                            indoor: data.indoor,
                            public: data.public,
                        });
                if let Some(loc) = new_loc {
                    world.locations.entry(destination).or_insert(loc);
                }
            }

            session.event_bus.emit(
                "text-log",
                &TextLogPayload {
                    source: "system".to_string(),
                    content: narration,
                },
            );

            handle_look(session).await;

            let world = session.world.lock().await;
            session.event_bus.emit(
                "world-update",
                &parish_core::ipc::snapshot_from_world(&world),
            );
        }
        MovementResult::AlreadyHere => {
            session.event_bus.emit(
                "text-log",
                &TextLogPayload {
                    source: "system".to_string(),
                    content: "Sure, you're already standing right here.".to_string(),
                },
            );
        }
        MovementResult::NotFound(name) => {
            let world = session.world.lock().await;
            let exits = format_exits(world.player_location, &world.graph);
            session.event_bus.emit(
                "text-log",
                &TextLogPayload {
                    source: "system".to_string(),
                    content: format!(
                        "You haven't the faintest notion how to reach \"{}\". {}",
                        name, exits
                    ),
                },
            );
        }
    }
}

/// Renders the current location description and exits.
async fn handle_look(session: &Arc<GameSession>) {
    let world = session.world.lock().await;
    let npc_manager = session.npc_manager.lock().await;

    let desc = if let Some(loc_data) = world.current_location_data() {
        let tod = world.clock.time_of_day();
        let weather = world.weather.to_string();
        let npc_display: Vec<String> = npc_manager
            .npcs_at(world.player_location)
            .iter()
            .map(|n| npc_manager.display_name(n).to_string())
            .collect();
        let npc_names: Vec<&str> = npc_display.iter().map(|s| s.as_str()).collect();
        render_description(loc_data, tod, &weather, &npc_names)
    } else {
        world.current_location().description.clone()
    };

    let exits = format_exits(world.player_location, &world.graph);

    session.event_bus.emit(
        "text-log",
        &TextLogPayload {
            source: "system".to_string(),
            content: format!("{}\n{}", desc, exits),
        },
    );
}

/// Routes input to the NPC at the player's location, or shows idle message.
async fn handle_npc_conversation(
    raw: String,
    session: &Arc<GameSession>,
    state: &Arc<ServerState>,
) {
    let (npc_name, npc_id, system_prompt, context, queue) = {
        let world = session.world.lock().await;
        let mut npc_manager = session.npc_manager.lock().await;
        let queue = state.inference_queue.lock().await;

        let npcs_here = npc_manager.npcs_at(world.player_location);
        let npc = npcs_here.first().cloned().cloned();

        if let (Some(npc), Some(q)) = (npc, queue.clone()) {
            let display = npc_manager.display_name(&npc).to_string();
            let id = npc.id;
            let other_npcs: Vec<&parish_core::npc::Npc> =
                npcs_here.into_iter().filter(|n| n.id != npc.id).collect();
            let system = ticks::build_enhanced_system_prompt(&npc, false);
            let ctx = ticks::build_enhanced_context(&npc, &world, &raw, &other_npcs);
            npc_manager.mark_introduced(id);
            (Some(display), Some(id), Some(system), Some(ctx), Some(q))
        } else {
            (None, None, None, None, None)
        }
    };

    let (Some(npc_name), Some(_npc_id), Some(system_prompt), Some(context), Some(queue)) =
        (npc_name, npc_id, system_prompt, context, queue)
    else {
        let idle_messages = [
            "The wind stirs, but nothing else.",
            "Only the sound of a distant crow.",
            "A dog barks somewhere beyond the hill.",
            "The clouds shift. The parish carries on.",
        ];
        let idx = REQUEST_ID.fetch_add(1, Ordering::SeqCst) as usize % idle_messages.len();
        session.event_bus.emit(
            "text-log",
            &TextLogPayload {
                source: "system".to_string(),
                content: idle_messages[idx].to_string(),
            },
        );
        return;
    };

    let model = {
        let config = state.config.lock().await;
        config.model_name.clone()
    };
    let req_id = REQUEST_ID.fetch_add(1, Ordering::SeqCst);

    session
        .event_bus
        .emit("loading", &LoadingPayload { active: true });

    let (token_tx, token_rx) = mpsc::unbounded_channel::<String>();

    let display_label = capitalize_first(&npc_name);
    session.event_bus.emit(
        "text-log",
        &TextLogPayload {
            source: display_label,
            content: String::new(),
        },
    );

    match queue
        .send(
            req_id,
            model,
            context,
            Some(system_prompt),
            Some(token_tx),
            None,
        )
        .await
    {
        Ok(mut response_rx) => {
            let bus = &session.event_bus;

            let stream_handle = tokio::spawn({
                let session_clone = Arc::clone(session);
                async move {
                    crate::streaming::stream_npc_response(&session_clone.event_bus, token_rx).await
                }
            });

            let full_response = loop {
                match response_rx.try_recv() {
                    Ok(resp) => {
                        let _ = stream_handle.await;
                        break Some(resp);
                    }
                    Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    }
                    Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                        break None;
                    }
                }
            };

            let hints = if let Some(resp) = full_response {
                if resp.error.is_some() {
                    tracing::warn!("Inference error: {:?}", resp.error);
                    vec![]
                } else {
                    let parsed = parse_npc_stream_response(&resp.text);
                    parsed
                        .metadata
                        .map(|m| m.language_hints)
                        .unwrap_or_default()
                }
            } else {
                vec![]
            };

            bus.emit("stream-end", &StreamEndPayload { hints });
        }
        Err(e) => {
            tracing::error!("Failed to submit inference request: {}", e);
        }
    }

    session
        .event_bus
        .emit("loading", &LoadingPayload { active: false });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submit_input_request_deserialization() {
        let json = r#"{"text": "go to church"}"#;
        let req: SubmitInputRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.text, "go to church");
    }
}
