use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    str::FromStr,
};

use config::{Environment, File};
use non_empty_string::NonEmptyString;
use serde::Deserialize;
use smart_default::SmartDefault;
use tracing::instrument;
use url::Url;

#[derive(SmartDefault, Deserialize)]
#[serde(default)]
pub struct Config {
    pub githost: GitHostChoice,

    #[default("intellectual-bot-for-github[bot]".try_into().unwrap())]
    pub bot_name: NonEmptyString,

    pub app_id: usize,

    pub installation_id: usize,

    #[default(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))]
    pub webhook_addr: IpAddr,

    #[default(8099)]
    pub webhook_server_port: u16,

    pub pem_rsa_key_path: PathBuf,

    pub llm: LlmChoice,

    #[default(Url::from_str("https://api.openai.com/v1").unwrap())]
    pub llm_api_base_url: Url,

    #[default("gpt-4o-mini".try_into().unwrap())]
    pub llm_model: NonEmptyString,

    #[default(false)]
    pub allow_list: bool,

    #[default(vec![])]
    pub features: Vec<NonEmptyString>,
}

#[derive(Clone, Deserialize, SmartDefault)]
pub enum LlmChoice {
    #[default]
    OpenAi,
    // :)
}

#[derive(Clone, SmartDefault, Deserialize)]
pub enum GitHostChoice {
    #[default]
    GitHub,
    // :)
}

const BOT_CONFIG_FILE: &'static str = "config.yaml";
const BOT_CONFIG_PREFIX: &'static str = "GIB";

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("unable to setup config sources")]
    BuildError(#[source] config::ConfigError),

    #[error("unable to deserialize config settings")]
    DeserializationError(#[source] config::ConfigError),
}

pub type Result<T> = std::result::Result<T, ConfigError>;

impl Config {
    #[instrument]
    pub fn build() -> Result<Self> {
        let builder = config::Config::builder()
            .add_source(
                File::with_name(
                    &std::env::var(&format!("{}_CONFIG_FILE", BOT_CONFIG_PREFIX))
                        .unwrap_or(BOT_CONFIG_FILE.into()),
                )
                .required(false),
            )
            .add_source(Environment::with_prefix(BOT_CONFIG_PREFIX))
            .build()
            .map_err(ConfigError::BuildError)?;

        builder
            .try_deserialize()
            .map_err(ConfigError::DeserializationError)
    }
}
