use solana_client::client_error::ClientError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolanaError {
    #[error("SolanaError - ClientError: {0}")]
    ClientError(#[from] ClientError),
    #[error("SolanaError - IoError: {0}")]
    IoError(#[from] std::io::Error),
    #[error("SolanaError - CacheInsertError: {0}")]
    CacheInsertError(String),
    #[error("SolanaError - RpcError: {0}")]
    RpcError(String),
}
