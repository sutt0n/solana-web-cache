use thiserror::Error;

use crate::{solana::error::SolanaError, web::error::WebError};

#[derive(Error, Debug)]
pub enum CliError {
    #[error("{0}")]
    WebError(#[from] WebError),
    #[error("{0}")]
    SolanaError(#[from] SolanaError),
}
