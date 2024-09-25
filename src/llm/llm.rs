use async_trait::async_trait;
use non_empty_string::NonEmptyString;
use smart_default::SmartDefault;

use super::messages::{AiMessage, ChatMessage};

#[derive(SmartDefault)]
pub struct CompletionParameters {
    #[default(1.0)]
    pub temperature: f32,
}

#[async_trait]
pub trait Llm {
    type Error;

    async fn complete(
        &self,
        system_message: &NonEmptyString, // NOTE: Using impl AsRef<str> introduces too much problems.
        chat: Vec<ChatMessage>,
        params: &CompletionParameters,
    ) -> Result<AiMessage, Self::Error>;
}
