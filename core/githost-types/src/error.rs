use std::error::Error;

pub type Result<T> = std::result::Result<T, GitHostError>;

#[derive(Debug, thiserror::Error)]
pub enum GitHostError {
    #[error("Unknown error")]
    Unknown(#[from] Box<dyn Error>),
}
