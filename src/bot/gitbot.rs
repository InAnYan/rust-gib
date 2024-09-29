use serde::Deserialize;
use tracing::instrument;

use crate::{
    githost::{events::GitEvent, host::GitHost},
    llm::llm::Llm,
};

use super::{
    bot_features::{BotFeatures, FeaturesConfig},
    errors::Result,
};

#[derive(Deserialize)]
pub struct GitBotConfig {
    features: FeaturesConfig,
}

pub struct GitBot<G, L> {
    features: BotFeatures<G, L>,
}

impl<G: GitHost + Clone, L: Llm + Clone> GitBot<G, L> {
    pub async fn build(
        config: GitBotConfig,
        githost: G,
        llm: L,
    ) -> Result<Self, G::Error, L::Error> {
        Ok(Self {
            features: BotFeatures::build_from_config(config.features, githost, llm).await?,
        })
    }

    pub fn build_raw(features: BotFeatures<G, L>) -> Self {
        Self { features }
    }

    #[instrument(skip(self))]
    pub async fn process_event(&self, event: &GitEvent) -> Result<(), G::Error, L::Error> {
        self.features.process_event(event).await
    }
}
