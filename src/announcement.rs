use std::{collections::HashMap, net::SocketAddr};

use axum::{
    extract::{
        ConnectInfo, Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{AppState, utils};

#[derive(FromRow, Serialize, Deserialize, Clone)]
pub struct RawDetection {
    pub id: i64,
    pub author: String,
    pub time: i64,
    pub tags: String,
}
impl RawDetection {
    pub fn convert(&self) -> Detection {
        Detection {
            id: self.id,
            author: self.author.clone(),
            time: self.time,
            tags: urlencoding::decode(&self.tags)
                .unwrap()
                .split(",")
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Detection {
    pub id: i64,
    pub author: String,
    pub time: i64,
    pub tags: Vec<String>,
}

pub async fn socket_handler(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
    ws: WebSocketUpgrade,
    ConnectInfo(_): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, query))
}

async fn handle_socket(socket: WebSocket, mut state: AppState, _: HashMap<String, String>) {
    let mut rx = state.tx.subscribe();

    let (mut sender, mut receiver) = socket.split();

    let mut receive_task = tokio::spawn(async move {
        // server receives this

        while let Some(Ok(msg)) = receiver.next().await {
            if (utils::get_time() - state.last_sent) > 2 {
                state.last_sent = utils::get_time();
                if let Ok(mut candidate) =
                    serde_json::from_str::<RawDetection>(&msg.to_text().unwrap().to_string())
                {
                    candidate.id = utils::get_time();
                    candidate.time = utils::get_time();

                    let _ = sqlx::query("insert into detection values ($1, $2, $3, $4);")
                        .bind(candidate.id)
                        .bind(candidate.author.clone())
                        .bind(candidate.time)
                        .bind(candidate.tags.clone())
                        .execute(&state.db)
                        .await;
                    let _ = state
                        .tx
                        .send(serde_json::to_string(&candidate.clone()).unwrap());
                }
            }
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

pub async fn history(State(state): State<AppState>) -> impl IntoResponse {
    serde_json::to_string(
        &sqlx::query_as::<_, RawDetection>("select * from detection")
            .fetch_all(&state.db)
            .await
            .unwrap()
            .iter()
            .map(|x| x.convert())
            .collect::<Vec<Detection>>(),
    )
    .unwrap()
    .into_response()
}
