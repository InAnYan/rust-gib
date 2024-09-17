use std::sync::Arc;

use nonempty::NonEmpty;
use tokio::sync::Mutex;

use crate::{
    features::feature_type::GitBotFeature,
    githost::{events::GitEvent, host::GitHost},
};

use super::errors::{GitBotError, Result};

pub struct GitBot {
    host: Arc<Mutex<dyn GitHost + Send + Sync>>,
    features: NonEmpty<Arc<Mutex<dyn GitBotFeature + Send>>>,
}

impl GitBot {
    pub fn new(
        host: Arc<Mutex<dyn GitHost + Send + Sync>>,
        features: NonEmpty<Arc<Mutex<dyn GitBotFeature + Send>>>,
    ) -> Self {
        Self { host, features }
    }

    pub async fn process_event(&self, event: &GitEvent) -> Result<()> {
        let mut res = Vec::new();

        for feature in self.features.iter() {
            match feature
                .lock()
                .await
                .process_event(event, self.host.clone())
                .await
            {
                Ok(()) => {}
                Err(e) => res.push((feature.lock().await.get_name(), e)),
            }
        }

        if res.is_empty() {
            Ok(())
        } else {
            Err(GitBotError::FeaturesError(res))
        }
    }
}
