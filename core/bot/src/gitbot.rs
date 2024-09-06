use std::sync::Arc;

use feature_types::{error::FeatureError, feature::GitBotFeature};
use githost_types::{event::GitEvent, githost::GitHost};
use non_empty_string::NonEmptyString;
use nonempty::nonzero::NonEmpty;
use tokio::sync::Mutex;

pub struct GitBot {
    host: Arc<Mutex<dyn GitHost + Send>>,
    features: NonEmpty<Box<dyn GitBotFeature>>,
}

impl GitBot {
    pub fn new(
        host: Arc<Mutex<dyn GitHost + Send>>,
        features: NonEmpty<Box<dyn GitBotFeature>>,
    ) -> Self {
        Self { host, features }
    }

    pub async fn process_event(
        &self,
        event: &GitEvent,
    ) -> Result<(), Vec<(NonEmptyString, FeatureError)>> {
        let mut res = Vec::new();

        for feature in self.features.iter() {
            match feature.process_event(event, self.host.clone()).await {
                Ok(()) => {}
                Err(e) => res.push((feature.get_name(), e)),
            }
        }

        if res.is_empty() {
            Ok(())
        } else {
            Err(res)
        }
    }
}
