use std::error::Error;

pub type Result<T> = std::result::Result<T, GitHostError>;

#[derive(Debug, thiserror::Error)]
pub enum GitHostError {
    #[error("unable to read secret key")]
    KeyRead(#[source] Box<dyn Error>),

    #[error("cannot access GitHub API or wrong request")]
    RequestError(#[source] Box<dyn Error>),

    #[error("invalid format of the response")]
    InvalidFormat,

    #[error("unknown error")]
    Unknown(#[source] Box<dyn Error>),
}
