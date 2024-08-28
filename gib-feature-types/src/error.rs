use std::error::Error;

pub type Result<T> = std::result::Result<T, FeatureError>;

#[derive(Debug, thiserror::Error)]
pub enum FeatureError {
    #[error("Unknown error")]
    Unknown(#[from] Box<dyn Error>),
}
