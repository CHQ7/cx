use axum::{
    extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::api::routes::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamMessage {
    pub event: String,
    pub data: serde_json::Value,
}

/// Handle WebSocket connections for streaming output
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    _state: axum::extract::State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    // Create a channel for streaming messages
    let (tx, mut rx) = mpsc::channel::<StreamMessage>(100);

    // Spawn a task to send messages to the client
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if sender.send(WsMessage::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            WsMessage::Text(text) => {
                // Echo back for now
                let response = StreamMessage {
                    event: "echo".to_string(),
                    data: serde_json::Value::String(text),
                };
                let _ = tx.send(response).await;
            }
            WsMessage::Close(_) => break,
            _ => {}
        }
    }

    send_task.abort();
}
