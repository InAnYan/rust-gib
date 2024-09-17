use crate::{githost::errors::GitHostError, llm::errors::LlmError};

#[derive(Debug, thiserror::Error)]
pub enum GitFeatureError {
    #[error("error from LLM")]
    LlmError(#[from] LlmError),

    #[error("unable to perform Git host action")]
    GitHostError(#[from] GitHostError),

    #[error("unknown error ocurred")]
    UnknownError(#[source] Box<dyn std::error::Error + Send>),
}

pub type Result<T> = std::result::Result<T, GitFeatureError>;
