pub type Result<T> = std::result::Result<T, GitHostError>;

#[derive(Debug, thiserror::Error)]
pub enum GitHostError {
    #[error("unable to decode secret key")]
    SecretKeyDecodeError,

    #[error("cannot access Git host API or wrong request")]
    GitHostRequestError,

    #[error("unable to bind webhook server to the supplied address")]
    WebhookServerBindError,

    #[error("internal server error for webhooks")]
    WebhookServerError,

    #[error("invalid format of the API response")]
    ApiResponseInvalidFormatError,

    #[error("unknown error")]
    UnknownError,
}
