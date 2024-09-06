use std::sync::Arc;

use async_trait::async_trait;
use githost_types::{event::GitEvent, githost::GitHost};
use non_empty_string::NonEmptyString;
use tokio::sync::Mutex;

use crate::error::Result;

#[async_trait]
pub trait GitBotFeature {
    async fn process_event(
        &self,
        event: &GitEvent,
        host: Arc<Mutex<dyn GitHost + Send>>,
    ) -> Result<()>;
    fn get_name(&self) -> NonEmptyString;
}
