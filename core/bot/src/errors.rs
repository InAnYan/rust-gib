use feature_types::error::FeatureError;
use non_empty_string::NonEmptyString;

pub type Result<T> = std::result::Result<T, GitBotError>;

#[derive(thiserror::Error, Debug)]
pub enum GitBotError {
    #[error("Some features returned error")]
    FeaturesError(Vec<(NonEmptyString, FeatureError)>),
}
