//! Shared application state and event bus for the web server.
//!
//! Supports per-session game isolation: each visitor gets their own world,
//! NPC manager, and event bus while sharing the inference pipeline.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{Mutex, RwLock, broadcast};

use parish_core::inference::InferenceQueue;
use parish_core::inference::openai_client::OpenAiClient;
use parish_core::npc::manager::NpcManager;
use parish_core::world::{LocationId, WorldState};

/// Unique session identifier (UUID v4 string).
pub type SessionId = String;

/// Per-visitor isolated game state.
///
/// Each browser session gets its own world, NPC manager, and event bus
/// so players don't interfere with each other.
pub struct GameSession {
    /// The game world (clock, player position, graph, weather).
    pub world: Mutex<WorldState>,
    /// NPC manager (all NPCs, tier assignment, schedule ticking).
    pub npc_manager: Mutex<NpcManager>,
    /// Broadcast channel for pushing events to this session's WebSocket.
    pub event_bus: EventBus,
    /// When this session was created.
    pub created_at: Instant,
    /// Last time the session was accessed (for idle cleanup).
    pub last_activity: Mutex<Instant>,
}

impl GameSession {
    /// Updates the last activity timestamp to now.
    pub async fn touch(&self) {
        *self.last_activity.lock().await = Instant::now();
    }
}

/// Manages all active game sessions and shared resources.
///
/// The inference pipeline (queue + clients) is shared across sessions
/// since it's stateless. Only game state is per-session.
pub struct SessionManager {
    /// Active sessions keyed by UUID.
    sessions: RwLock<HashMap<SessionId, Arc<GameSession>>>,
    /// Path to data directory (for loading world/NPC templates).
    data_dir: PathBuf,
    /// Starting location ID for new sessions.
    start_location: LocationId,
    /// Maximum concurrent sessions allowed.
    max_sessions: usize,
}

impl SessionManager {
    /// Creates a new session manager.
    pub fn new(data_dir: PathBuf, start_location: LocationId, max_sessions: usize) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            data_dir,
            start_location,
            max_sessions,
        }
    }

    /// Creates a new game session by loading fresh state from data files.
    ///
    /// Returns the session ID, or `None` if the session limit is reached.
    pub async fn create_session(&self) -> Option<(SessionId, Arc<GameSession>)> {
        let mut sessions = self.sessions.write().await;
        if sessions.len() >= self.max_sessions {
            return None;
        }

        let id = uuid::Uuid::new_v4().to_string();

        let world =
            WorldState::from_parish_file(&self.data_dir.join("parish.json"), self.start_location)
                .unwrap_or_else(|e| {
                    tracing::warn!(
                        "Failed to load parish.json for session: {}. Using default.",
                        e
                    );
                    WorldState::new()
                });

        let mut npc_manager = NpcManager::load_from_file(&self.data_dir.join("npcs.json"))
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to load npcs.json for session: {}. No NPCs.", e);
                NpcManager::new()
            });
        npc_manager.assign_tiers(world.player_location, &world.graph);

        let now = Instant::now();
        let session = Arc::new(GameSession {
            world: Mutex::new(world),
            npc_manager: Mutex::new(npc_manager),
            event_bus: EventBus::new(256),
            created_at: now,
            last_activity: Mutex::new(now),
        });

        sessions.insert(id.clone(), Arc::clone(&session));
        Some((id, session))
    }

    /// Looks up a session by ID.
    pub async fn get(&self, id: &str) -> Option<Arc<GameSession>> {
        self.sessions.read().await.get(id).cloned()
    }

    /// Removes sessions idle longer than `timeout`.
    ///
    /// Returns the number of sessions removed.
    pub async fn remove_idle(&self, timeout: std::time::Duration) -> usize {
        let mut sessions = self.sessions.write().await;
        let mut to_remove = Vec::new();

        for (id, session) in sessions.iter() {
            let last = *session.last_activity.lock().await;
            if last.elapsed() > timeout {
                to_remove.push(id.clone());
            }
        }

        for id in &to_remove {
            tracing::info!("Cleaning up idle session {}", id);
            sessions.remove(id);
        }

        to_remove.len()
    }

    /// Returns the current number of active sessions.
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
}

/// Top-level server state passed to all Axum route handlers.
///
/// Holds the session manager and shared inference resources.
pub struct ServerState {
    /// Per-session game state manager.
    pub sessions: SessionManager,
    /// Inference request queue shared across all sessions.
    pub inference_queue: Mutex<Option<InferenceQueue>>,
    /// Local LLM client (shared, stateless HTTP client).
    pub client: Mutex<Option<OpenAiClient>>,
    /// Cloud LLM client for dialogue (shared).
    pub cloud_client: Mutex<Option<OpenAiClient>>,
    /// Mutable runtime configuration.
    pub config: Mutex<GameConfig>,
}

/// Mutable runtime configuration for provider, model, and cloud settings.
pub struct GameConfig {
    /// Display name of the current base provider.
    pub provider_name: String,
    /// Base URL for the current provider API.
    pub base_url: String,
    /// API key for the current provider.
    pub api_key: Option<String>,
    /// Model name for NPC dialogue inference.
    pub model_name: String,
    /// Cloud provider name for dialogue.
    pub cloud_provider_name: Option<String>,
    /// Cloud model name for dialogue.
    pub cloud_model_name: Option<String>,
    /// Cloud API key.
    pub cloud_api_key: Option<String>,
    /// Cloud base URL.
    pub cloud_base_url: Option<String>,
    /// Whether improv craft mode is enabled.
    pub improv_enabled: bool,
    /// Per-category provider name overrides (Dialogue=0, Simulation=1, Intent=2).
    pub category_provider: [Option<String>; 3],
    /// Per-category model name overrides.
    pub category_model: [Option<String>; 3],
    /// Per-category API key overrides.
    pub category_api_key: [Option<String>; 3],
    /// Per-category base URL overrides.
    pub category_base_url: [Option<String>; 3],
}

impl GameConfig {
    /// Returns the array index for a category.
    pub fn cat_idx(cat: parish_core::config::InferenceCategory) -> usize {
        use parish_core::config::InferenceCategory;
        match cat {
            InferenceCategory::Dialogue => 0,
            InferenceCategory::Simulation => 1,
            InferenceCategory::Intent => 2,
        }
    }
}

/// A JSON-serializable server event pushed to WebSocket clients.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ServerEvent {
    /// Event name (e.g. "stream-token", "text-log").
    pub event: String,
    /// JSON payload for this event.
    pub payload: serde_json::Value,
}

/// Broadcast channel for server-push events.
///
/// Events emitted here are forwarded to all connected WebSocket clients.
pub struct EventBus {
    tx: broadcast::Sender<ServerEvent>,
}

impl EventBus {
    /// Creates a new event bus with the given channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Sends an event to all subscribers. Returns the number of receivers.
    pub fn send(&self, event: ServerEvent) -> usize {
        self.tx.send(event).unwrap_or(0)
    }

    /// Emits a named event with a serializable payload.
    pub fn emit<T: serde::Serialize>(&self, event_name: &str, payload: &T) {
        if let Ok(value) = serde_json::to_value(payload) {
            self.send(ServerEvent {
                event: event_name.to_string(),
                payload: value,
            });
        }
    }

    /// Creates a new receiver for this bus.
    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_bus_send_and_subscribe() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();
        bus.emit("test-event", &serde_json::json!({"key": "value"}));
        let event = rx.try_recv().unwrap();
        assert_eq!(event.event, "test-event");
        assert_eq!(event.payload["key"], "value");
    }

    #[test]
    fn event_bus_no_subscribers() {
        let bus = EventBus::new(16);
        // No subscribers — should not panic
        let count = bus.send(ServerEvent {
            event: "orphan".to_string(),
            payload: serde_json::Value::Null,
        });
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn session_manager_create_and_get() {
        let dir = std::env::temp_dir();
        let mgr = SessionManager::new(dir, LocationId(1), 10);
        let (id, _session) = mgr.create_session().await.unwrap();
        assert!(mgr.get(&id).await.is_some());
        assert!(mgr.get("nonexistent").await.is_none());
        assert_eq!(mgr.session_count().await, 1);
    }

    #[tokio::test]
    async fn session_manager_respects_limit() {
        let dir = std::env::temp_dir();
        let mgr = SessionManager::new(dir, LocationId(1), 1);
        let _first = mgr.create_session().await.unwrap();
        assert!(mgr.create_session().await.is_none());
    }
}
