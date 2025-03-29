pub mod error;

use crate::{cache::Cache, solana::SolanaClientTrait, web};
use anyhow::Result;
use error::CliError;
use std::sync::Arc;
use tracing::{error, info};

pub async fn run(
    web_port: u64,
    solana: Arc<dyn SolanaClientTrait + Send + Sync>,
    cache: Arc<Cache>,
) -> Result<(), CliError> {
    let (send, mut receive) = tokio::sync::mpsc::channel::<Result<(), CliError>>(1);
    let mut handles = vec![];

    let get_blocks_chunk_size = 10;

    info!("Starting web server");
    let web_send = send.clone();
    let web_cache = Arc::clone(&cache);
    let web_solana = Arc::clone(&solana);
    handles.push(tokio::spawn(async move {
        web_send
            .try_send(web::run_web(web_port, &web_cache, web_solana).await.map_err(CliError::WebError))
            .unwrap_or_else(|e| error!("Failed sending web task result: {:?}", e));
    }));

    info!("Starting Solana slot polling");
    let slot_send = send.clone();
    let solana_slot_client = Arc::clone(&solana);
    handles.push(tokio::spawn(async move {
        let solana = solana_slot_client;
        slot_send
            .try_send(solana.poll_for_latest_slot().await.map_err(CliError::SolanaError))
            .unwrap_or_else(|e| error!("Failed sending slot polling result: {:?}", e));
    }));

    info!("Starting Solana cache fulfillment");
    let cache_send = send.clone();
    let solana_cache_client = Arc::clone(&solana);
    handles.push(tokio::spawn(async move {
        let solana = solana_cache_client;
        cache_send
            .try_send(
                solana.contiguously_get_confirmed_blocks(get_blocks_chunk_size).await.map_err(CliError::SolanaError),
            )
            .unwrap_or_else(|e| error!("Failed sending cache fulfillment result: {:?}", e));
    }));

    let reason = receive.recv().await.expect("Didn't receive msg");
    for handle in handles {
        handle.abort();
        info!("Task handle aborted");
    }

    reason
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cache::Cache, solana::MockSolanaClientTrait};
    use std::sync::Arc;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_cli() {
        let cache = Arc::new(Cache::new(1000));

        let mut mock_solana = MockSolanaClientTrait::new();

        mock_solana.expect_poll_for_latest_slot().returning(|| Ok(()));

        mock_solana.expect_contiguously_get_confirmed_blocks().returning(|_| Ok(()));

        mock_solana.expect_is_slot_confirmed().returning(|_| true);

        let mock_solana = Arc::new(mock_solana);

        let result = run(3000, mock_solana, cache).await;

        assert!(result.is_ok(), "CLI run should exit successfully");
    }
}
