use gib_feature_types::{error::FeatureError, feature::GitBotFeature};
use gib_githost_types::event::GitEvent;
use non_empty_string::NonEmptyString;
use nonempty::nonzero::NonEmpty;

pub struct GitBot<H> {
    host: H,
    features: NonEmpty<Box<dyn GitBotFeature<H>>>,
}

impl<H> GitBot<H> {
    pub fn new(host: H, features: NonEmpty<Box<dyn GitBotFeature<H>>>) -> Self {
        Self { host, features }
    }

    pub async fn process_event(
        &mut self,
        event: &GitEvent,
    ) -> Result<(), Vec<(NonEmptyString, FeatureError)>> {
        let mut res = Vec::new();

        for feature in self.features.iter_mut() {
            match feature.process_event(event, &self.host).await {
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
