use std::sync::Arc;

use async_trait::async_trait;
use non_empty_string::NonEmptyString;
use tokio::sync::Mutex;

use crate::{
    errors::Result,
    githost::{event::GitEvent, githost::GitHost},
};

#[async_trait]
pub trait GitBotFeature {
    async fn process_event(
        &self,
        event: &GitEvent,
        host: Arc<Mutex<dyn GitHost + Send + Sync>>,
    ) -> Result<()>;

    fn get_name(&self) -> NonEmptyString;
}
