use std::{env::VarError, fmt::Debug, sync::Arc};

use gib::{
    bot::{errors::GitBotError, gitbot::GitBot},
    config::{
        make_github_host, make_improve_feature, make_label_feature, make_openai_llm, Config,
        ConfigError, FeaturesConfig, GitHostChoice, LlmChoice,
    },
    features::{errors::GitFeatureError, feature_type::GitBotFeature},
    githost::{
        errors::GitHostError,
        events::GitEvent,
        host::GitHost,
        impls::github::{self},
    },
    llm::{errors::LlmError, llm::Llm},
};
use nonempty::NonEmpty;
use secrecy::SecretString;
use tokio::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Mutex,
    },
    task::{JoinError, JoinHandle},
};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const GIT_EVENT_CHANNEL_BUFFER_SIZE: usize = 2;

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("unable to read config")]
    ConfigError(#[from] ConfigError),

    #[error("unable to read secret key")]
    SecretKeyReadError(#[source] std::io::Error),

    #[error("error from Git host")]
    GitHostError(#[from] GitHostError),

    #[error("unable to retrieve LLM API KEY from environment")]
    NoLlmApiKey(#[source] VarError),

    #[error("error from LLM")]
    LlmError(#[from] LlmError),

    #[error("error from bot feature")]
    FeatureError(#[from] GitFeatureError),

    #[error("you must specify at least one feature")]
    NoFeaturesSelected,

    #[error("webhook server configuration should be present for the selected Git host")]
    NoWebhookConfiguration,

    #[error("error from Git bot")]
    GitBotError(#[from] GitBotError),

    #[error("unable to join threads")]
    ThreadJoinError(#[from] JoinError),
}

type Result<T> = std::result::Result<T, MainError>;

const GIB_LLM_API_KEY_VAR: &'static str = "GIB_LLM_API_KEY";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let config = Config::build()?;

    let githost: Arc<Mutex<dyn GitHost + Send + Sync>> = match config.githost {
        GitHostChoice::GitHub(config) => Arc::new(Mutex::new(make_github_host(config).await?)),
    };

    let llm: Arc<Mutex<dyn Llm + Send>> = match config.llm {
        LlmChoice::OpenAi(config) => Arc::new(Mutex::new(make_openai_llm(
            config,
            SecretString::new(std::env::var(GIB_LLM_API_KEY_VAR).map_err(MainError::NoLlmApiKey)?),
        )?)),
    };

    let features = build_features(config.features, llm).await?;

    let bot = GitBot::new(githost, features);

    let (events_send, mut events_receive): (Sender<GitEvent>, Receiver<GitEvent>) =
        channel(GIT_EVENT_CHANNEL_BUFFER_SIZE);

    if let Some(webhook_config) = config.webhook_server {
        let webhook_server_join = tokio::spawn(github::webhook_server::listen_to_events(
            events_send,
            webhook_config.addr,
            webhook_config.port,
        ));

        let bot_join: JoinHandle<std::result::Result<(), GitBotError>> = tokio::spawn(async move {
            while let Some(event) = events_receive.recv().await {
                bot.process_event(&event).await?;
            }

            Ok(())
        });

        let (webhook_exit, bot_exit) = tokio::join!(webhook_server_join, bot_join);
        webhook_exit??;
        bot_exit??;

        Ok(())
    } else {
        Err(MainError::NoWebhookConfiguration)
    }
}

async fn build_features(
    config: FeaturesConfig,
    llm: Arc<Mutex<dyn Llm + Send>>,
) -> Result<NonEmpty<Arc<Mutex<dyn GitBotFeature + Send>>>> {
    let mut vec: Vec<Arc<Mutex<dyn GitBotFeature + Send>>> = vec![];

    if let Some(improve_feature) = config.improve_feature {
        vec.push(Arc::new(Mutex::new(
            make_improve_feature(improve_feature, llm.clone()).await?,
        )));
    }

    if let Some(label_feature) = config.label_feature {
        vec.push(Arc::new(Mutex::new(
            make_label_feature(label_feature, llm.clone()).await?,
        )));
    }

    NonEmpty::from_vec(vec).ok_or(MainError::NoFeaturesSelected)
}
