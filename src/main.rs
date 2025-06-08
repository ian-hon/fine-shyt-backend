use std::net::SocketAddr;

use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use sqlx::{Pool, Sqlite, SqlitePool, sqlite::SqliteConnectOptions};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

use crate::announcement::RawDetection;

mod extractor_error;
mod utils;

mod announcement;

pub async fn not_implemented_yet() -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        "not implemented yet chill".to_string(),
    )
        .into_response()
}

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Sqlite>,
    pub tx: broadcast::Sender<String>,
    pub last_sent: i64,
}

#[tokio::main]
async fn main() {
    println!(
        "{}",
        serde_json::to_string(&RawDetection {
            id: 0,
            author: "me".to_string(),
            time: 10,
            tags: "abc".to_string(),
        })
        .unwrap()
    );

    let app = Router::new()
        .route("/", get(|| async { "fine-shyt at your service" }))
        .route("/ws", get(announcement::socket_handler))
        .route("/history", get(announcement::history))
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_origin(Any)
                .allow_headers(Any),
        )
        .with_state(AppState {
            db: SqlitePool::connect_with(SqliteConnectOptions::new().filename("db.sqlite3"))
                .await
                .unwrap(),
            tx: broadcast::channel(1024).0,
            last_sent: 0,
        });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8531")
        .await
        .unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
