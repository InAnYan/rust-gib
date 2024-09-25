use std::{env::VarError, fmt::Debug};

use gib::{
    bot::gitbot::{GitBot, GitBotError},
    config::{Config, ConfigError, GitHostChoice, LlmChoice},
    githost::{
        events::GitEvent,
        impls::github::{self, errors::GithubError, github_host::GithubHost},
    },
    llm::impls::openai_llm::{OpenAiLlm, OpenAiLlmError},
};
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    task::{JoinError, JoinHandle},
};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const GIT_EVENT_CHANNEL_BUFFER_SIZE: usize = 2;

#[derive(Debug, thiserror::Error)]
pub enum MainError<GE, LE> {
    #[error("unable to read config")]
    ConfigError(#[from] ConfigError),

    #[error("unable to read secret key")]
    SecretKeyReadError(#[source] std::io::Error),

    #[error("error from Git host")]
    GitHostError(#[source] GE),

    #[error("unable to retrieve LLM API KEY from environment")]
    NoLlmApiKey(#[source] VarError),

    #[error("error from LLM")]
    LlmError(#[source] LE),

    #[error("you must specify at least one feature")]
    NoFeaturesSelected,

    #[error("webhook server configuration should be present for the selected Git host")]
    NoWebhookConfiguration,

    #[error("error from Git bot")]
    GitBotError(#[from] GitBotError<GE, LE>),

    #[error("unable to join threads")]
    ThreadJoinError(#[from] JoinError),
}

type Result<T, GE, LE> = std::result::Result<T, MainError<GE, LE>>;

#[tokio::main]
async fn main() -> Result<(), GithubError, OpenAiLlmError> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let config = Config::build()?;

    let githost = match config.githost {
        GitHostChoice::Github(config) => GithubHost::build(config)
            .await
            .map_err(MainError::GitHostError)?,
    };

    let llm = match config.llm {
        LlmChoice::OpenAi(config) => OpenAiLlm::build(config).map_err(MainError::LlmError)?,
    };

    let bot = GitBot::build(config.bot, githost, llm).await?;

    let (events_send, mut events_receive): (Sender<GitEvent>, Receiver<GitEvent>) =
        channel(GIT_EVENT_CHANNEL_BUFFER_SIZE);

    if let Some(webhook_config) = config.webhook_server {
        let webhook_server_join = tokio::spawn(github::webhook_server::listen_to_events(
            events_send,
            webhook_config,
        ));

        let bot_join: JoinHandle<
            std::result::Result<(), GitBotError<GithubError, OpenAiLlmError>>,
        > = tokio::spawn(async move {
            while let Some(event) = events_receive.recv().await {
                bot.process_event(&event).await?;
            }

            Ok(())
        });

        let (webhook_exit, bot_exit) = tokio::join!(webhook_server_join, bot_join);
        webhook_exit?.map_err(MainError::GitHostError)?;
        bot_exit??;

        Ok(())
    } else {
        Err(MainError::NoWebhookConfiguration)
    }
}
