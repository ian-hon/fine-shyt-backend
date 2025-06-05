use std::{collections::HashMap, net::SocketAddr};

use axum::{
    extract::{
        ConnectInfo, Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};

use crate::AppState;

pub async fn socket_handler(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, query))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, query: HashMap<String, String>) {
    let mut rx = state.tx.subscribe();

    let (mut sender, mut receiver) = socket.split();

    let mut receive_task = tokio::spawn(async move {
        // server receives this

        while let Some(Ok(msg)) = receiver.next().await {
            let _ = state.tx.send(msg.into_text().unwrap());
        }
    });

    let mut send_task = tokio::spawn(async move {
        // server sends this
        while let Ok(msg) = rx.recv().await {
            let _ = sender.send(Message::Text(msg)).await;
        }
    });

    tokio::select! {
        _ = &mut send_task => receive_task.abort(),
        _ = &mut receive_task => send_task.abort(),
    };
}
