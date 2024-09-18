use std::{net::IpAddr, path::PathBuf, sync::Arc};

use config::{Environment, File};
use non_empty_string::NonEmptyString;
use secrecy::{SecretString, SecretVec};
use serde::Deserialize;
use tokio::{
    fs::{read, read_to_string},
    sync::Mutex,
};
use tracing::instrument;
use url::Url;

use crate::{
    features::{
        errors::GitFeatureError, improve_feature::GitImproveFeature, label_feature::GitLabelFeature,
    },
    githost::{errors::GitHostError, impls::github::github_host::GitHubHost},
    llm::{errors::LlmError, impls::openai_llm::OpenAiLlm, llm::Llm},
};

#[derive(Deserialize)]
pub struct Config {
    pub githost: GitHostChoice,
    pub webhook_server: Option<WebhookServerConfig>,
    pub llm: LlmChoice,
    pub features: FeaturesConfig,
}

#[derive(Deserialize)]
pub enum GitHostChoice {
    GitHub(GitHubHostConfig),
}

#[derive(Deserialize)]
pub struct GitHubHostConfig {
    pub bot_name: NonEmptyString,
    pub app_id: usize,
    pub installation_id: usize,
    pub pem_rsa_key_path: PathBuf,
}

pub async fn make_github_host(config: GitHubHostConfig) -> Result<GitHubHost, GitHostError> {
    GitHubHost::build(
        config.bot_name,
        config.app_id as u64,
        config.installation_id as u64,
        SecretVec::new(
            read(config.pem_rsa_key_path)
                .await
                .map_err(GitHostError::SecretKeyFileOpenError)?,
        ),
    )
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

#[derive(Deserialize)]
pub struct OpenAiLlmConfig {
    pub api_base_url: Url,
    pub model_name: NonEmptyString,
}

pub fn make_openai_llm(
    config: OpenAiLlmConfig,
    api_key: SecretString,
) -> Result<OpenAiLlm, LlmError> {
    Ok(OpenAiLlm::new(
        config.api_base_url,
        api_key,
        config.model_name,
    ))
}

#[derive(Deserialize)]
pub struct FeaturesConfig {
    pub improve_feature: Option<ImproveFeatureConfig>,
    pub label_feature: Option<LabelFeatureConfig>,
}

#[derive(Deserialize)]
pub struct ImproveFeatureConfig {
    system_message_template_path: PathBuf,
    user_message_template_path: PathBuf,
    temperature: f32,
}

pub async fn make_improve_feature(
    config: ImproveFeatureConfig,
    llm: Arc<Mutex<dyn Llm + Send>>,
) -> Result<GitImproveFeature, GitFeatureError> {
    Ok(GitImproveFeature::build(
        llm,
        config.temperature,
        read_to_string(config.system_message_template_path.clone())
            .await
            .map_err(|e| {
                GitFeatureError::TemplateReadError(config.system_message_template_path.clone(), e)
            })?
            .try_into()
            .map_err(|_| {
                GitFeatureError::TemplateEmptyError(config.system_message_template_path)
            })?,
        read_to_string(config.user_message_template_path.clone())
            .await
            .map_err(|e| {
                GitFeatureError::TemplateReadError(config.user_message_template_path.clone(), e)
            })?
            .try_into()
            .map_err(|_| GitFeatureError::TemplateEmptyError(config.user_message_template_path))?,
    )?)
}

#[derive(Deserialize)]
pub struct LabelFeatureConfig {
    system_message_template_path: PathBuf,
    user_message_template_path: PathBuf,
    temperature: f32,
}

pub async fn make_label_feature(
    config: LabelFeatureConfig,
    llm: Arc<Mutex<dyn Llm + Send>>,
) -> Result<GitLabelFeature, GitFeatureError> {
    Ok(GitLabelFeature::build(
        llm,
        config.temperature,
        read_to_string(config.system_message_template_path.clone())
            .await
            .map_err(|e| {
                GitFeatureError::TemplateReadError(config.system_message_template_path.clone(), e)
            })?
            .try_into()
            .map_err(|_| {
                GitFeatureError::TemplateEmptyError(config.system_message_template_path)
            })?,
        read_to_string(config.user_message_template_path.clone())
            .await
            .map_err(|e| {
                GitFeatureError::TemplateReadError(config.user_message_template_path.clone(), e)
            })?
            .try_into()
            .map_err(|_| GitFeatureError::TemplateEmptyError(config.user_message_template_path))?,
    )?)
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

impl Config {
    #[instrument]
    pub fn build() -> Result<Self, ConfigError> {
        let builder = config::Config::builder()
            .add_source(File::with_name(
                &std::env::var(&format!("{}_CONFIG_FILE", BOT_CONFIG_PREFIX))
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
