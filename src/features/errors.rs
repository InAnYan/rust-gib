use crate::{githost::errors::GitHostError, llm::errors::LlmError};
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum GitFeatureError {
    #[error("unable to read template file")]
    TemplateReadError(PathBuf, #[source] std::io::Error),

    #[error("template is empty")]
    TemplateEmptyError(PathBuf),

    #[error("unable to add template")]
    TemplateAddError,

    #[error("unable to render the template")]
    TemplateRenderError,

    #[error("rendered template is empty")]
    TemplateRenderIsEmptyError,

    #[error("error from LLM")]
    LlmError(#[from] LlmError),

    #[error("unable to perform Git host action")]
    GitHostError(#[from] GitHostError),

    #[error("unknown error ocurred")]
    UnknownError(#[source] Box<dyn std::error::Error + Send>),
}

pub type Result<T> = std::result::Result<T, GitFeatureError>;
