pub mod error;

use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use error::WebError;
use tokio::sync::Mutex;

use crate::{cache::Cache, solana::SolanaClientTrait};

#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<Cache>,
    pub solana: Arc<Mutex<dyn SolanaClientTrait + Send + Sync>>,
}

pub async fn run_web(
    port: u64,
    cache: &Arc<Cache>,
    solana: Arc<Mutex<dyn SolanaClientTrait + Send + Sync>>,
) -> anyhow::Result<(), WebError> {
    let app_state = AppState { cache: Arc::clone(cache), solana };

    let app = Router::new().route("/isSlotConfirmed/{slot}", get(slot_get)).with_state(app_state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await.unwrap();

    println!("Starting web server on port {}", port);

    axum::serve(listener, app).await.map_err(WebError::IoError)
}

async fn slot_get(State(app_state): State<AppState>, Path(slot): Path<u64>) -> impl IntoResponse {
    let cache = &app_state.cache;

    if let Some(_) = cache.get(&slot).await {
        return StatusCode::OK;
    }

    let is_slot_confirmed = app_state.solana.lock().await.is_slot_confirmed(slot).await;
    if !is_slot_confirmed {
        return StatusCode::NOT_FOUND;
    }

    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cache::Cache, solana::MockSolanaClientTrait};
    use axum::{extract::State, http::StatusCode};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_slot_get_found_in_cache() {
        let cache = Arc::new(Cache::new(1000));
        cache.insert(10, 10).await.unwrap();

        let mock_solana = MockSolanaClientTrait::new();

        let state = AppState { cache, solana: Arc::new(Mutex::new(mock_solana)) };

        let response = slot_get(State(state), Path(10)).await;
        assert_eq!(response.into_response().status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_slot_get_confirmed_by_solana() {
        let cache = Arc::new(Cache::new(1000));

        let mut mock_solana = MockSolanaClientTrait::new();
        mock_solana.expect_is_slot_confirmed().withf(|&slot| slot == 10).returning(|_| true);

        let state = AppState { cache, solana: Arc::new(Mutex::new(mock_solana)) };

        let response = slot_get(State(state), Path(10)).await;
        assert_eq!(response.into_response().status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_slot_get_not_confirmed() {
        let cache = Arc::new(Cache::new(1000));

        let mut mock_solana = MockSolanaClientTrait::new();
        mock_solana.expect_is_slot_confirmed().withf(|&slot| slot == 10).returning(|_| false);

        let state = AppState { cache, solana: Arc::new(Mutex::new(mock_solana)) };

        let response = slot_get(State(state), Path(10)).await;
        assert_eq!(response.into_response().status(), StatusCode::NOT_FOUND);
    }
}
