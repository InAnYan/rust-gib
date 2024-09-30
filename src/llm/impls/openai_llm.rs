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
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use url::Url;

use crate::llm::{
    llm_trait::{CompletionParameters, Llm},
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
    pub api_key_env_var: NonEmptyString,
}

#[derive(Clone)]
pub struct OpenAiLlm {
    client: Client<OpenAIConfig>,
    model_name: NonEmptyString,
}

impl OpenAiLlm {
    pub fn build(config: OpenAiLlmConfig) -> Result<Self, OpenAiLlmError> {
        let api_key =
            std::env::var(config.api_key_env_var.as_str()).map_err(OpenAiLlmError::ApiKeyNotSet)?;

        Self::build_raw(
            config.api_base_url,
            config.model_name,
            SecretString::new(api_key),
        )
    }

    pub fn build_raw(
        api_base_url: Url,
        model_name: NonEmptyString,
        api_key: SecretString,
    ) -> Result<Self, OpenAiLlmError> {
        let url: String = api_base_url.into();

        let client = Client::with_config(
            OpenAIConfig::new()
                .with_api_base(match url.strip_suffix("/") {
                    Some(s) => s.to_string(),
                    None => url,
                })
                .with_api_key(api_key.expose_secret()),
        );

        Ok(Self { client, model_name })
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
                .first()
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use non_empty_string::NonEmptyString;
    use secrecy::SecretString;
    use serde_json::json;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::llm::{
        impls::openai_llm::OpenAiLlm,
        llm_trait::{CompletionParameters, Llm},
        messages::UserMessage,
    };

    #[tokio::test]
    async fn openai_completion() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
              "id": "chatcmpl-123",
              "object": "chat.completion",
              "created": 1677652288,
              "model": "eliza",
              "system_fingerprint": "fp_44709d6fcb",
              "choices": [{
                "index": 0,
                "message": {
                  "role": "assistant",
                  "content": "assistant",
                },
                "logprobs": null,
                "finish_reason": "stop"
              }],
              "usage": {
                "prompt_tokens": 9,
                "completion_tokens": 12,
                "total_tokens": 21,
                "completion_tokens_details": {
                  "reasoning_tokens": 0
                }
              }
            }
            )))
            .mount(&mock_server)
            .await;

        let llm = OpenAiLlm::build_raw(
            mock_server.uri().as_str().try_into().unwrap(),
            "eliza".try_into().unwrap(),
            SecretString::new("42".into()),
        )
        .unwrap();

        let response = llm
            .complete(
                &NonEmptyString::from_str("system").unwrap(),
                vec![UserMessage::from_str("user").unwrap().into()],
                &CompletionParameters { temperature: 1.0 },
            )
            .await
            .unwrap();

        assert_eq!(response.as_str(), "assistant");
    }
}
