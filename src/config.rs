use std::net::IpAddr;

use config::{Environment, File};
use serde::Deserialize;
use tracing::instrument;

use crate::{
    bot::gitbot::GitBotConfig, githost::impls::github::github_host::GithubConfig,
    llm::impls::openai_llm::OpenAiLlmConfig,
};

#[derive(Deserialize)]
pub struct Config {
    pub githost: GitHostChoice,
    pub webhook_server: Option<WebhookServerConfig>,
    pub llm: LlmChoice,
    pub bot: GitBotConfig,
}

#[derive(Deserialize)]
pub enum GitHostChoice {
    Github(GithubConfig),
}

#[derive(Deserialize)]
pub struct WebhookServerConfig {
    pub addr: IpAddr,
    pub port: u16,
}

#[derive(Deserialize)]
pub enum LlmChoice {
    OpenAi(OpenAiLlmConfig),
}

const BOT_CONFIG_FILE: &str = "config.yaml";
const BOT_CONFIG_PREFIX: &str = "GIB";

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("unable to setup config sources")]
    BuildError(#[source] config::ConfigError),

    #[error("unable to deserialize config settings")]
    DeserializationError(#[source] config::ConfigError),
}

impl Config {
    #[instrument]
    pub fn build() -> Result<Self, ConfigError> {
        let builder = config::Config::builder()
            .add_source(File::with_name(
                &std::env::var(format!("{}_CONFIG_FILE", BOT_CONFIG_PREFIX))
                    .unwrap_or(BOT_CONFIG_FILE.into()),
            ))
            .add_source(Environment::with_prefix(BOT_CONFIG_PREFIX))
            .build()
            .map_err(ConfigError::BuildError)?;

        builder
            .try_deserialize()
            .map_err(ConfigError::DeserializationError)
    }
}
