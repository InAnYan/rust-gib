use std::sync::Arc;

use async_trait::async_trait;
use feature_types::{error::Result, feature::GitBotFeature};
use githost_types::{event::GitEvent, githost::GitHost};
use llm_types::llm::Llm;
use non_empty_string::NonEmptyString;
use tokio::sync::Mutex;

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
        todo!()
    }

    fn get_name(&self) -> NonEmptyString {
        // I'm afraid of this `unwrap`...
        "label-issues".try_into().unwrap()
    }
}
