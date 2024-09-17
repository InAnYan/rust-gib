use std::{collections::HashSet, process::exit, sync::Arc};

use gib::{
    bot::{errors::GitBotError, gitbot::GitBot},
    config::{Config, ConfigError},
    features::{
        feature_type::GitBotFeature, improve_feature::GitImproveFeature,
        label_feature::GitLabelFeature,
    },
    githost::{
        errors::GitHostError,
        events::GitEvent,
        impls::github::{self, github_host::GitHubHost},
    },
    llm::impls::openai_llm::OpenAiLlm,
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
    task::JoinHandle,
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

    #[error("error from Git bot")]
    GitBotError(#[from] GitBotError),
}

#[tokio::main]
async fn main() -> Result<(), MainError> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let config = Config::build()?;

    let githost = GitHubHost::build(
        config.bot_name,
        config.app_id as u64,
        config.installation_id as u64,
        SecretVec::new(
            read(config.pem_rsa_key_path)
                .await
                .map_err(MainError::SecretKeyReadError)?,
        ),
    )
    .expect("unable to create GitHub host");

    let githost = Arc::new(Mutex::new(githost));

    let llm = OpenAiLlm::new(
        config.llm_api_base_url,
        SecretString::new(
            std::env::var("GIB_LLM_API_KEY")
                .expect("environment variable GIB_LLM_API_KEY must be set"),
        ),
        config.llm_model,
    );

    let llm = Arc::new(Mutex::new(llm));

    let features_to_add = if config.allow_list {
        config.features
    } else {
        let mut available = vec![
            "improve-issues".try_into().unwrap(),
            "label-issues".try_into().unwrap(),
        ];
        available.retain(|f| !config.features.contains(f));
        available
    }
    .into_iter()
    .collect::<HashSet<NonEmptyString>>();

    info!("Running features: {:?}", &features_to_add);

    let mut features: Vec<Arc<Mutex<dyn GitBotFeature + Send>>> = vec![];

    for feature in features_to_add {
        match feature.as_str() {
            "improve-issues" => {
                features.push(Arc::new(Mutex::new(GitImproveFeature::new(llm.clone()))))
            }

            "label-issues" => {
                features.push(Arc::new(Mutex::new(GitLabelFeature::new(llm.clone()))))
            }

            _ => {
                eprintln!("Error: unknown feature '{}'", feature);
                exit(1)
            }
        }
    }

    let features = match NonEmpty::from_vec(features) {
        Some(r) => r,
        None => {
            eprintln!("Error: no features selected.");
            exit(1)
        }
    };

    let bot = GitBot::new(githost, features);

    let (events_send, mut events_receive): (Sender<GitEvent>, Receiver<GitEvent>) =
        channel(GIT_EVENT_CHANNEL_BUFFER_SIZE);

    let webhook_server_join = github::webhook_server::listen_to_events(
        events_send,
        config.webhook_addr,
        config.webhook_server_port,
    );

    let bot_join: JoinHandle<Result<(), GitBotError>> = tokio::spawn(async move {
        while let Some(event) = events_receive.recv().await {
            bot.process_event(&event).await?;
        }

        Ok(())
    });

    let (r1, r2) = tokio::join!(webhook_server_join, bot_join);
    r1?;
    r2.unwrap()?; // FIX: unwrap-crap.

    Ok(())
}
