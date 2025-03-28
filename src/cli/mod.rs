pub mod error;

use std::sync::Arc;

use error::CliError;

use crate::{cache::Cache, solana::SolanaClient, web};
use anyhow::{Context, Result};

pub async fn run() -> Result<(), CliError> {
    let (send, mut receive) = tokio::sync::mpsc::channel::<Result<(), CliError>>(1);
    let mut handles = vec![];

    // todo: pull into config
    let get_blocks_chunk_size = 10;
    let max_size = 1000;

    let cache = Arc::new(Cache::new(max_size));
    let mut solana = SolanaClient::init(&Arc::clone(&cache)).await?;

    println!("Starting web server");
    let web_send = send.clone();
    let solana_client_web = solana.clone();
    handles.push(tokio::spawn(async move {
        let _ = web_send.try_send(
            web::run_web(3000, &cache, solana_client_web)
                .await
                .map_err(|e| CliError::WebError(e)),
        );
    }));

    println!("Starting Solana slot polling");
    let solana_slot_send = send.clone();
    let mut solana_client_slot = solana.clone();
    handles.push(tokio::spawn(async move {
        let _ = solana_slot_send.try_send(
            solana_client_slot
                .poll_for_latest_slot()
                .await
                .map_err(|e| CliError::SolanaError(e)),
        );
    }));

    println!("Starting Solana cache fulfillment");
    let solana_cache_send = send.clone();
    handles.push(tokio::spawn(async move {
        let _ = solana_cache_send.try_send(
            solana
                .contiguously_get_confirmed_blocks(get_blocks_chunk_size)
                .await
                .map_err(|e| CliError::SolanaError(e)),
        );
    }));

    let reason = receive.recv().await.expect("Didn't receive msg");
    for handle in handles {
        handle.abort();
    }

    reason
}
