use non_empty_string::NonEmptyString;

pub type Result<T> = std::result::Result<T, GibError>;

#[derive(Debug, thiserror::Error)]
pub enum GibError {
    #[error("unable to send request to LLM")]
    LlmSendingError,

    #[error("LLM returned message in the wrong format")]
    LlmFormatError,

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

    #[error("some features have returned errors")]
    FeaturesError(Vec<(NonEmptyString, Box<GibError>)>),

    #[error("unknown error")]
    UnknownError,
}