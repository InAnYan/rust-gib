use async_trait::async_trait;
use gib_githost_types::event::GitEvent;
use non_empty_string::NonEmptyString;

use crate::error::Result;

#[async_trait]
pub trait GitBotFeature<H> {
    async fn process_event(&mut self, event: &GitEvent, host: &H) -> Result<()>;
    fn get_name(&self) -> NonEmptyString;
}
