use std::{iter::once, str::FromStr};

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use async_trait::async_trait;
use non_empty_string::NonEmptyString;
use secrecy::{ExposeSecret, SecretString};
use url::Url;

use crate::{
    errors::{GibError, Result},
    llm::{
        llm::{CompletionParameters, Llm},
        messages::{AiMessage, ChatMessage},
    },
};

pub struct OpenAiLlm {
    client: Client<OpenAIConfig>,
    model: NonEmptyString,
}

impl OpenAiLlm {
    pub fn new(api_base_url: Url, api_key: SecretString, model: NonEmptyString) -> Self {
        Self {
            client: Client::with_config(
                OpenAIConfig::new()
                    .with_api_base(api_base_url)
                    .with_api_key(api_key.expose_secret()),
            ),
            model,
        }
    }
}

#[async_trait]
impl Llm for OpenAiLlm {
    async fn complete(
        &self,
        system_message: &str,
        chat: Vec<ChatMessage>,
        params: &CompletionParameters,
    ) -> Result<AiMessage> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(self.model.as_str())
            .messages(
                once(make_system_message(system_message))
                    .chain(chat.into_iter().map(chat_message_to_openai))
                    .collect::<Result<Vec<ChatCompletionRequestMessage>>>()?,
            )
            .temperature(params.temperature)
            .build()
            .map_err(|e| GibError::UnknownError(e.into()))?; // I really have no idea why this may
                                                             // fail...

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| GibError::LlmSendingError(e.into()))?;

        Ok(AiMessage::from_str(
            &response
                .choices
                .get(0)
                .ok_or(GibError::ApiResponseInvalidFormatError)?
                .message
                .content
                .clone()
                .ok_or(GibError::ApiResponseInvalidFormatError)?,
        )
        .map_err(|_| GibError::ApiResponseInvalidFormatError)?)
    }
}

fn chat_message_to_openai(msg: ChatMessage) -> Result<ChatCompletionRequestMessage> {
    match msg {
        ChatMessage::UserMessage(content) => ChatCompletionRequestUserMessageArgs::default()
            .content(content.as_str())
            .build()
            .map_err(|e| GibError::UnknownError(e.into()))
            .map(|m| m.into()),

        ChatMessage::AiMessage(content) => ChatCompletionRequestAssistantMessageArgs::default()
            .content(content.as_str())
            .build()
            .map_err(|e| GibError::UnknownError(e.into()))
            .map(|m| m.into()),
    }
}

fn make_system_message(content: impl AsRef<str>) -> Result<ChatCompletionRequestMessage> {
    ChatCompletionRequestSystemMessageArgs::default()
        .content(content.as_ref())
        .build()
        .map_err(|e| GibError::UnknownError(e.into()))
        .map(|m| m.into())
}
