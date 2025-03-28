//pub mod app;
pub mod cache;
pub mod cli;
pub mod solana;
pub mod web;

use std::sync::Arc;

use cache::Cache;
use solana::SolanaClient;
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    tracing::info!("Application starting...");

    let cache = Arc::new(Cache::new(1000));
    let solana_client = Arc::new(SolanaClient::init(Arc::clone(&cache)).await);

    if let Err(e) = cli::run(solana_client, cache).await {
        tracing::error!("Application exited with error: {:?}", e);
    }

    tracing::info!("Application exiting gracefully.");

    Ok(())
}
