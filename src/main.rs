//pub mod app;
pub mod cache;
pub mod cli;
pub mod solana;
pub mod web;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::run().await?;

    Ok(())
}
