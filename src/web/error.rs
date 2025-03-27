use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebError {
    #[error("WebError - IoError: {0}")]
    IoError(#[from] std::io::Error),
}
