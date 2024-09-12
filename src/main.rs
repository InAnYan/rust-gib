use std::{collections::HashSet, process::exit, sync::Arc};

use clap::Parser;
use cli_args::CliArgs;
use gib::{
    bot::{feature_type::GitBotFeature, gitbot::GitBot},
    errors::GibError,
    features::{improve_feature::GitImproveFeature, label_feature::GitLabelFeature},
    githost::{
        event::GitEvent,
        impls::github::{github_githost::GitHubHost, webhook_server::listen_to_events},
    },
    llm::impls::openai_llm::OpenAiLlm,
};
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

mod cli_args;

const GIT_EVENT_CHANNEL_BUFFER_SIZE: usize = 2;

#[tokio::main]
async fn main() -> Result<(), GibError> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let args = CliArgs::parse();

    let githost = GitHubHost::build(
        args.app_id as u64,
        args.installation_id as u64,
        SecretVec::new(
            read(args.pem_rsa_key_path)
                .await
                .map_err(|_| GibError::SecretKeyDecodeError)?,
        ),
    )
    .expect("unable to create GitHub host");

    let githost = Arc::new(Mutex::new(githost));

    let llm = OpenAiLlm::new(
        args.llm_api_base_url,
        SecretString::new(
            std::env::var("GIB_LLM_API_KEY")
                .expect("environment variable GIB_LLM_API_KEY must be set"),
        ),
        args.llm_model,
    );

    let llm = Arc::new(Mutex::new(llm));

    let features_to_add = if args.allow_list {
        args.features
    } else {
        let mut available = vec![
            "improve-issues".try_into().unwrap(),
            "label-issues".try_into().unwrap(),
        ];
        available.retain(|f| !args.features.contains(f));
        available
    }
    .into_iter()
    .collect::<HashSet<NonEmptyString>>();

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

    let webhook_server_join =
        listen_to_events(events_send, args.webhook_addr, args.webhook_server_port);

    let bot_join: JoinHandle<Result<(), GibError>> = tokio::spawn(async move {
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
