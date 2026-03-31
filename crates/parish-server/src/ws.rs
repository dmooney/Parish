//! WebSocket handler for server-push events.
//!
//! Each connected client gets a WebSocket scoped to their session,
//! receiving JSON-encoded [`ServerEvent`] frames from that session's
//! [`EventBus`].

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::routes::SessionQuery;
use crate::state::{GameSession, ServerState};

/// Upgrades the HTTP connection to a WebSocket scoped to a session.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
    Query(q): Query<SessionQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let session = state
        .sessions
        .get(&q.session)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    session.touch().await;
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, session)))
}

/// Handles a single WebSocket connection.
///
/// Subscribes to the session's [`EventBus`] and forwards each event as a
/// JSON text frame until the client disconnects or the bus is dropped.
async fn handle_socket(mut socket: WebSocket, session: Arc<GameSession>) {
    let mut rx = session.event_bus.subscribe();
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
                        // Touch session on client activity
                        session.touch().await;
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
