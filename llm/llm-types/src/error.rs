use std::error::Error;

pub type Result<T> = std::result::Result<T, LlmError>;

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("Wrong API URL passed")]
    UrlNotFound,

    #[error("Reached connection timeout")]
    Timeout,

    #[error("No money left on the account")]
    NoMoney,

    #[error("API key is wrong")]
    WrongApiKey,

    #[error("Count of tokens is more than context window can handle")]
    ContextOverflow,

    #[error("Unknown error")]
    Unknown(#[from] Box<dyn Error>),
}
