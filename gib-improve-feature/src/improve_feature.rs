use async_trait::async_trait;
use gib_feature_types::{error::Result, feature::GitBotFeature};
use gib_githost_types::{event::GitEvent, githost::GitHost};
use non_empty_string::NonEmptyString;

pub struct GitImproveFeature {}

#[async_trait]
impl<H: GitHost> GitBotFeature<H> for GitImproveFeature {
    async fn process_event(&mut self, event: &GitEvent, host: &H) -> Result<()> {
        todo!()
    }

    fn get_name(&self) -> NonEmptyString {
        // I'm afraid of this `unwrap`...
        "improve-issues".try_into().unwrap()
    }
}
