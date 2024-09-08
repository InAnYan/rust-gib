use async_trait::async_trait;
use smart_default::SmartDefault;

use crate::errors::Result;

use super::messages::{AiMessage, ChatMessage};

#[derive(SmartDefault)]
pub struct CompletionParameters {
    #[default(1.0)]
    pub temperature: f32,
}

#[async_trait]
pub trait Llm {
    async fn complete(
        &self,
        system_message: &str, // NOTE: Using impl AsRef<str> introduces too much problems.
        chat: Vec<ChatMessage>,
        params: &CompletionParameters,
    ) -> Result<AiMessage>;
}
