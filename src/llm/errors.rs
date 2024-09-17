#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("unable to send request to LLM")]
    RequestError,

    #[error("LLM API returned message in the wrong format")]
    FormatError,

    #[error("unknown error")]
    UnknownError,
}

pub type Result<T> = std::result::Result<T, LlmError>;
