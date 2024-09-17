use non_empty_string::NonEmptyString;

use crate::features::errors::GitFeatureError;

#[derive(Debug, thiserror::Error)]
pub enum GitBotError {
    #[error("some features returned error")]
    FeaturesError(Vec<(NonEmptyString, GitFeatureError)>),
}

pub type Result<T> = std::result::Result<T, GitBotError>;
