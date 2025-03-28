//pub mod app;
pub mod cache;
pub mod cli;
pub mod solana;
pub mod web;

use std::sync::Arc;

use anyhow::Result;
use cache::Cache;
use solana::SolanaClient;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    let cache = Arc::new(Cache::new(1000));
    let solana_client = Arc::new(Mutex::new(SolanaClient::init(Arc::clone(&cache)).await));

    cli::run(solana_client, cache).await?;
    Ok(())
}
