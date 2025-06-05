use std::net::SocketAddr;

use axum::{
    Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

mod extractor_error;
mod hermes_error;
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
    pub tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "fine-shyt at your service" }))
        .route("/message/ws", get(announcement::socket_handler))
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_origin(Any)
                .allow_headers(Any),
        )
        .with_state(AppState {
            tx: broadcast::channel(1024).0,
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
