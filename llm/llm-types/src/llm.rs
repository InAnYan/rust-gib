use async_trait::async_trait;

use crate::{
    error::Result,
    messages::{AiMessage, ChatMessage},
};

pub struct CompletionParameters {
    pub temperature: f32,
}

#[async_trait]
pub trait Llm {
    async fn complete(
        &self,
        system_message: &str, // It's easier to pass `&str` instead of impl AsRef<str>.
        chat: Vec<ChatMessage>,
        params: &CompletionParameters,
    ) -> Result<AiMessage>;
}
