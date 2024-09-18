use std::{collections::HashSet, fmt::Debug, process::exit, str::FromStr, sync::Arc};

use gib::{
    bot::{errors::GitBotError, gitbot::GitBot},
    config::{Config, ConfigError, GitHostChoice, LlmChoice},
    features::{
        feature_type::GitBotFeature, improve_feature::GitImproveFeature,
        label_feature::GitLabelFeature,
    },
    githost::{
        errors::GitHostError,
        events::GitEvent,
        host::GitHost,
        impls::github::{self, github_host::GitHubHost},
    },
    llm::{impls::openai_llm::OpenAiLlm, llm::Llm},
};
use log::info;
use non_empty_string::NonEmptyString;
use nonempty::NonEmpty;
use secrecy::{SecretString, SecretVec};
use tokio::{
    fs::read,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Mutex,
    },
    task::{JoinError, JoinHandle},
};
use tracing::instrument;
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

    #[error("specified feature does not exist")]
    UnknownFeature(NonEmptyString),

    #[error("you must specify at least one feature")]
    NoFeaturesSelected,

    #[error("error from Git bot")]
    GitBotError(#[from] GitBotError),

    #[error("unable to join threads")]
    ThreadJoinError(#[from] JoinError),
}

type Result<T> = std::result::Result<T, MainError>;

#[instrument(skip(config))]
async fn build_githost(config: &Config) -> Result<Arc<Mutex<dyn GitHost + Send + Sync>>> {
    match config.githost {
        GitHostChoice::GitHub => Ok(Arc::new(Mutex::new(GitHubHost::build(
            config.bot_name.clone(),
            config.app_id as u64,
            config.installation_id as u64,
            SecretVec::new(
                read(config.pem_rsa_key_path.clone())
                    .await
                    .map_err(MainError::SecretKeyReadError)?,
            ),
        )?))),
    }
}

#[instrument(skip(config))]
async fn build_llm(config: &Config) -> Result<Arc<Mutex<dyn Llm + Send>>> {
    match config.llm {
        LlmChoice::OpenAi => Ok(Arc::new(Mutex::new(OpenAiLlm::new(
            config.llm_api_base_url.clone(),
            SecretString::new(
                std::env::var("GIB_LLM_API_KEY")
                    .expect("environment variable GIB_LLM_API_KEY must be set"),
            ),
            config.llm_model.clone(),
        )))),
    }
}

// Strings there must be always non-empty.
const GIB_AVAILABLE_FEATURES: [&'static str; 2] = ["improve-issues", "label-issues"];

#[instrument]
fn compute_list_of_features(
    is_allow_list: bool,
    features: HashSet<NonEmptyString>,
) -> HashSet<NonEmptyString> {
    if is_allow_list {
        features
    } else {
        GIB_AVAILABLE_FEATURES
            .into_iter()
            .map(|s| NonEmptyString::from_str(s).unwrap()) // We expect that `GIB_AVAILABLE_FEATURES` contains only non-empty strings.
            .filter(|f| !features.contains(f))
            .collect()
    }
}

#[instrument(skip(llm))]
fn build_features(
    features_to_add: HashSet<NonEmptyString>,
    llm: Arc<Mutex<dyn Llm + Send>>,
) -> Result<Vec<Arc<Mutex<dyn GitBotFeature + Send>>>> {
    features_to_add
        .into_iter()
        .map(|feature_name| match feature_name.as_str() {
            "improve-issues" => Ok(Arc::new(Mutex::new(GitImproveFeature::new(llm.clone())))
                as Arc<Mutex<dyn GitBotFeature + Send>>),

            "label-issues" => Ok(Arc::new(Mutex::new(GitLabelFeature::new(llm.clone())))
                as Arc<Mutex<dyn GitBotFeature + Send>>),

            _ => Err(MainError::UnknownFeature(feature_name)),
        })
        .collect()
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let config = Config::build()?;

    let githost = build_githost(&config).await?;

    let llm = build_llm(&config).await?;

    let features =
        compute_list_of_features(config.allow_list, config.features.into_iter().collect());

    info!("Running features: {:?}", &features);

    let features = build_features(features, llm)?;

    let features = NonEmpty::from_vec(features).ok_or(MainError::NoFeaturesSelected)?;

    let bot = GitBot::new(githost, features);

    let (events_send, mut events_receive): (Sender<GitEvent>, Receiver<GitEvent>) =
        channel(GIT_EVENT_CHANNEL_BUFFER_SIZE);

    let webhook_server_join = tokio::spawn(github::webhook_server::listen_to_events(
        events_send,
        config.webhook_addr,
        config.webhook_server_port,
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
}
