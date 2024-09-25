use std::{iter::once, str::FromStr};

use async_openai::{
    config::OpenAIConfig,
    error::OpenAIError,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use async_trait::async_trait;
use non_empty_string::NonEmptyString;
use serde::Deserialize;
use url::Url;

use crate::llm::{
    llm::{CompletionParameters, Llm},
    messages::{AiMessage, ChatMessage},
};

#[derive(Debug, thiserror::Error)]
pub enum OpenAiLlmError {
    #[error("error from the underlying implementation crate")]
    ImplementationError(#[from] OpenAIError),

    #[error("API key environment variable is not set")]
    ApiKeyNotSet(#[source] std::env::VarError),

    #[error("LLM API returned message in the wrong format")]
    FormatError,
}

#[derive(Deserialize)]
pub struct OpenAiLlmConfig {
    pub api_base_url: Url,
    pub model_name: NonEmptyString,
    pub api_key_env_var_name: NonEmptyString,
}

#[derive(Clone)]
pub struct OpenAiLlm {
    client: Client<OpenAIConfig>,
    model_name: NonEmptyString,
}

impl OpenAiLlm {
    pub fn build(config: OpenAiLlmConfig) -> Result<Self, OpenAiLlmError> {
        let api_key = std::env::var(config.api_key_env_var_name.as_str())
            .map_err(OpenAiLlmError::ApiKeyNotSet)?;

        let client = Client::with_config(
            OpenAIConfig::new()
                .with_api_base(config.api_base_url)
                .with_api_key(api_key),
        );

        Ok(Self {
            client,
            model_name: config.model_name,
        })
    }
}

#[async_trait]
impl Llm for OpenAiLlm {
    type Error = OpenAiLlmError;

    async fn complete(
        &self,
        system_message: &NonEmptyString,
        chat: Vec<ChatMessage>,
        params: &CompletionParameters,
    ) -> Result<AiMessage, Self::Error> {
        let messages = once(make_system_message(system_message))
            .chain(chat.into_iter().map(chat_message_to_openai))
            .collect::<Result<Vec<ChatCompletionRequestMessage>, OpenAiLlmError>>()?;

        let request = CreateChatCompletionRequestArgs::default()
            .model(self.model_name.as_str())
            .messages(messages)
            .temperature(params.temperature)
            .build()?;

        let response = self.client.chat().create(request).await?;

        Ok(AiMessage::from_str(
            &response
                .choices
                .get(0)
                .ok_or(OpenAiLlmError::FormatError)?
                .message
                .content
                .clone()
                .ok_or(OpenAiLlmError::FormatError)?,
        )
        .map_err(|_| OpenAiLlmError::FormatError)?)
    }
}

fn chat_message_to_openai(
    msg: ChatMessage,
) -> Result<ChatCompletionRequestMessage, OpenAiLlmError> {
    Ok(match msg {
        ChatMessage::UserMessage(content) => ChatCompletionRequestUserMessageArgs::default()
            .content(content.as_str())
            .build()?
            .into(),

        ChatMessage::AiMessage(content) => ChatCompletionRequestAssistantMessageArgs::default()
            .content(content.as_str())
            .build()?
            .into(),
    })
}

fn make_system_message(
    content: impl AsRef<str>,
) -> Result<ChatCompletionRequestMessage, OpenAiLlmError> {
    Ok(ChatCompletionRequestSystemMessageArgs::default()
        .content(content.as_ref())
        .build()?
        .into())
}
