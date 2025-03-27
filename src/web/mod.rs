pub mod error;

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use error::WebError;

use crate::{cache::Cache, solana::SolanaClient};

#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<Cache>,
    pub solana: SolanaClient,
}

pub async fn run_web(
    port: u64,
    cache: &Arc<Cache>,
    solana: SolanaClient,
) -> anyhow::Result<(), WebError> {
    let app_state = AppState {
        cache: Arc::clone(cache),
        solana,
    };

    let app = Router::new()
        .route("/isSlotConfirmed/{slot}", get(slot_get))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    println!("Starting web server on port {}", port);

    axum::serve(listener, app)
        .await
        .map_err(|e| WebError::IoError(e.into()))
}

async fn slot_get(State(app_state): State<AppState>, Path(slot): Path<u64>) -> impl IntoResponse {
    let cache = &app_state.cache;

    if let Some(_) = cache.get(&slot).await {
        return StatusCode::OK;
    }

    let is_slot_confirmed = app_state.solana.is_slot_confirmed(slot).await;
    if !is_slot_confirmed {
        return StatusCode::NOT_FOUND;
    }

    StatusCode::OK
}
