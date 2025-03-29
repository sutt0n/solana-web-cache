//pub mod app;
pub mod cache;
pub mod cli;
pub mod solana;
pub mod web;

use std::sync::Arc;

use cache::Cache;
use clap::Parser;
use solana::SolanaClient;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Opts {
    #[clap(short, long)]
    pub api_key: String,

    #[clap(short, long)]
    pub port: u64,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    tracing::info!("Application starting...");

    let opts: Opts = Opts::parse();

    let cache = Arc::new(Cache::new(1000));
    let solana_client = Arc::new(SolanaClient::init(Arc::clone(&cache), opts.api_key).await);

    if let Err(e) = cli::run(opts.port, solana_client, cache).await {
        tracing::error!("Application exited with error: {:?}", e);
    }

    tracing::info!("Application exiting gracefully.");

    Ok(())
}
