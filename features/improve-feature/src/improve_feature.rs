use std::sync::Arc;

use async_trait::async_trait;
use feature_types::{error::Result, feature::GitBotFeature};
use githost_types::{event::GitEvent, githost::GitHost};
use llm_types::llm::Llm;
use non_empty_string::NonEmptyString;
use tokio::sync::Mutex;

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
        host: Arc<Mutex<dyn GitHost + Send>>,
    ) -> Result<()> {
        todo!()
    }

    fn get_name(&self) -> NonEmptyString {
        // I'm afraid of this `unwrap`...
        "improve-issues".try_into().unwrap()
    }
}
