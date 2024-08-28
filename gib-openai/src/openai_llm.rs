use async_trait::async_trait;
use gib_llm_types::{
    error::Result,
    llm::{CompletionParameters, Llm},
    messages::{AiMessage, ChatMessage},
};

pub struct OpenAiLlm {}

#[async_trait]
impl Llm for OpenAiLlm {
    async fn complete(
        &self,
        system_message: &str,
        chat: Vec<ChatMessage>,
        params: &CompletionParameters,
    ) -> Result<AiMessage> {
        todo!()
    }
}
