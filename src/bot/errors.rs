use super::features::{improve_feature::ImproveFeatureError, label_feature::LabelFeatureError};

#[derive(Debug, thiserror::Error)]
pub enum GitBotError<GE, LE> {
    #[error("issue-improve feature returned an error")]
    ImproveFeatureError(#[from] ImproveFeatureError<GE, LE>),

    #[error("issue-label feature returned an error")]
    LabelFeatureError(#[from] LabelFeatureError<GE, LE>),
}

pub type Result<T, GE, LE> = std::result::Result<T, GitBotError<GE, LE>>;
