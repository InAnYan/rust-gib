use async_trait::async_trait;

use crate::{action::GitAction, error::Result, event::GitEvent};

#[async_trait]
pub trait GitHost {
    async fn poll_events(&mut self) -> Result<Vec<GitEvent>>;
    async fn perform_action(&mut self, action: GitAction) -> Result<()>;
}
