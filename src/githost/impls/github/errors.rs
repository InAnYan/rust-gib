#[derive(Debug, thiserror::Error)]
pub enum GithubError {
    #[error("unable to open secret key file")]
    SecretKeyFileOpenError(#[source] std::io::Error),

    #[error("unable to decode secret key")]
    SecretKeyDecodeError(#[from] jsonwebtoken::errors::Error),

    #[error("error in the underlying implementation crate")]
    ImplementationError(#[from] octocrab::Error),

    #[error("cannot access Git host API or wrong request")]
    GitHostRequestError,

    #[error("unable to bind webhook server to the supplied address")]
    WebhookServerBindError(#[source] std::io::Error),

    #[error("internal server error of webhook server")]
    WebhookServerError(#[source] std::io::Error),

    #[error("invalid format of the API response")]
    ApiResponseInvalidFormatError,

    #[error("unknown error")]
    UnknownError,
}

pub type Result<T> = std::result::Result<T, GithubError>;
