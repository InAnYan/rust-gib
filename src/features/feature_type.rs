use std::sync::Arc;

use async_trait::async_trait;
use non_empty_string::NonEmptyString;
use tokio::sync::Mutex;

use crate::githost::{events::GitEvent, host::GitHost};

use super::errors::Result;

/// Whenever you develop a feature, remember that bot actions are also Git events, if you don't
/// properly filter out bot messages, you may have infinite recursion.
#[async_trait]
pub trait GitBotFeature {
    async fn process_event(
        &self,
        event: &GitEvent,
        host: Arc<Mutex<dyn GitHost + Send + Sync>>,
    ) -> Result<()>;

    fn get_name(&self) -> NonEmptyString;
}
