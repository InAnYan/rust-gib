use std::sync::Arc;

use async_trait::async_trait;
use gib_feature_types::{error::Result, feature::GitBotFeature};
use gib_githost_types::{event::GitEvent, githost::GitHost};
use gib_llm_types::llm::Llm;
use non_empty_string::NonEmptyString;
use tokio::sync::Mutex;

pub struct GitLabelFeature<L> {
    llm: Arc<Mutex<L>>,
}

#[async_trait]
impl<L: Llm + Send, H: GitHost> GitBotFeature<H> for GitLabelFeature<L> {
    async fn process_event(&mut self, event: &GitEvent, host: &H) -> Result<()> {
        todo!()
    }

    fn get_name(&self) -> NonEmptyString {
        // I'm afraid of this `unwrap`...
        "label-issues".try_into().unwrap()
    }
}
