use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use non_empty_string::NonEmptyString;
use tokio::sync::Mutex;

use crate::{
    bot::feature_type::GitBotFeature,
    errors::Result,
    githost::{event::GitEvent, githost::GitHost},
    llm::{
        llm::{CompletionParameters, Llm},
        messages::UserMessage,
    },
};

pub struct GitImproveFeature {
    llm: Arc<Mutex<dyn Llm + Send>>,
}

impl GitImproveFeature {
    pub fn new(llm: Arc<Mutex<dyn Llm + Send>>) -> Self {
        GitImproveFeature { llm }
    }
}

#[async_trait]
impl GitBotFeature for GitImproveFeature {
    async fn process_event(
        &self,
        event: &GitEvent,
        host: Arc<Mutex<dyn GitHost + Send + Sync>>,
    ) -> Result<()> {
        host.lock()
            .await
            .make_comment(
                event.repo_id,
                event.issue_id,
                self.llm
                    .lock()
                    .await
                    .complete(
                        "you are a bot".try_into().unwrap(),
                        vec![UserMessage::from_str("say something about improving")
                            .unwrap()
                            .into()],
                        &CompletionParameters::default(),
                    )
                    .await?
                    .as_str()
                    .try_into()
                    .unwrap(),
            )
            .await?;

        Ok(())
    }

    fn get_name(&self) -> NonEmptyString {
        // I'm afraid of this `unwrap`...
        "improve-issues".try_into().unwrap()
    }
}
