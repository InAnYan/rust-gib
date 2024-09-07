use std::sync::Arc;

use async_trait::async_trait;
use non_empty_string::NonEmptyString;
use tokio::sync::Mutex;

use crate::{
    bot::feature_type::GitBotFeature,
    errors::Result,
    githost::{event::GitEvent, githost::GitHost},
    llm::llm::Llm,
};

pub struct GitLabelFeature {
    llm: Arc<Mutex<dyn Llm + Send>>,
}

impl GitLabelFeature {
    pub fn new(llm: Arc<Mutex<dyn Llm + Send>>) -> Self {
        GitLabelFeature { llm }
    }
}

#[async_trait]
impl GitBotFeature for GitLabelFeature {
    async fn process_event(
        &self,
        event: &GitEvent,
        host: Arc<Mutex<dyn GitHost + Send>>,
    ) -> Result<()> {
        host.lock()
            .await
            .make_comment(
                event.repo_id,
                event.issue_id,
                "hi, I'm label feature".try_into().unwrap(),
            )
            .await?;

        Ok(())
    }

    fn get_name(&self) -> NonEmptyString {
        // I'm afraid of this `unwrap`...
        "label-issues".try_into().unwrap()
    }
}