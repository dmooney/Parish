//! WebSocket handler for server-push events.
//!
//! Each connected client gets a WebSocket that receives JSON-encoded
//! [`ServerEvent`] frames from the per-session [`EventBus`].
//!
//! # Authentication (#379)
//! The upgrade request must carry a short-lived HMAC session token as a
//! `?token=` query parameter.  Obtain one via `POST /api/session-init`.
//! Missing or invalid tokens are rejected with `401 Unauthorized`.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Extension, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::cf_auth::SessionToken;
use crate::state::AppState;

/// Upgrades the HTTP connection to a WebSocket.
///
/// Requires a valid `?token=` query parameter (issued by `POST /api/session-init`).
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    // #379 — validate session token before accepting the WS upgrade.
    let token = match params.get("token") {
        Some(t) => t.clone(),
        None => {
            tracing::warn!("ws_handler: rejected — missing ?token query param");
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    if let Err(e) = SessionToken::validate_full(&token) {
        tracing::warn!(error = %e, "ws_handler: rejected — invalid session token");
        return StatusCode::UNAUTHORIZED.into_response();
    }

    ws.on_upgrade(|socket| handle_socket(socket, state))
        .into_response()
}

/// Handles a single WebSocket connection.
///
/// Subscribes to the per-session [`EventBus`] and forwards each event as a
/// JSON text frame until the client disconnects or the bus is dropped.
async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.event_bus.subscribe();
    tracing::info!("WebSocket client connected");

    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Ok(server_event) => {
                        match serde_json::to_string(&server_event) {
                            Ok(json) => {
                                if socket.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to serialize event: {}", e);
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("WebSocket client lagged, dropped {} events", n);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(_)) => {
                        // Client messages are ignored (commands use REST)
                    }
                    _ => break,
                }
            }
        }
    }

    tracing::info!("WebSocket client disconnected");
}

#[cfg(test)]
mod tests {
    #[test]
    fn ws_module_compiles() {
        // Placeholder — real WebSocket tests require a running server
    }
}
