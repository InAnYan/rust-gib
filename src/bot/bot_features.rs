use serde::Deserialize;
use tracing::instrument;

use super::{
    errors::Result,
    features::{improve_feature::ImproveFeature, label_feature::LabelFeature},
};
use crate::{
    githost::{events::GitEvent, host::GitHost},
    llm::llm_trait::Llm,
};

use super::features::{improve_feature::ImproveFeatureConfig, label_feature::LabelFeatureConfig};

#[derive(Deserialize)]
pub struct FeaturesConfig {
    pub improve_feature: Option<ImproveFeatureConfig>,
    pub label_feature: Option<LabelFeatureConfig>,
}

pub struct BotFeatures<G, L> {
    pub improve_feature: Option<ImproveFeature<G, L>>,
    pub label_feature: Option<LabelFeature<G, L>>,
}

impl<G: GitHost + Clone, L: Llm + Clone> BotFeatures<G, L> {
    pub async fn build_from_config(
        config: FeaturesConfig,
        githost: G,
        llm: L,
    ) -> Result<Self, G::Error, L::Error> {
        Ok(Self {
            improve_feature: match config.improve_feature {
                Some(config) => Some(
                    ImproveFeature::build_from_config(config, githost.clone(), llm.clone()).await?,
                ),
                None => None,
            },

            label_feature: match config.label_feature {
                Some(config) => Some(
                    LabelFeature::build_from_config(config, githost.clone(), llm.clone()).await?,
                ),
                None => None,
            },
        })
    }
}

impl<G: GitHost, L: Llm> BotFeatures<G, L> {
    #[instrument(skip(self))]
    pub async fn process_event(&self, event: &GitEvent) -> Result<(), G::Error, L::Error> {
        if let Some(improve_feature) = &self.improve_feature {
            improve_feature.process_event(event).await?;
        }

        if let Some(label_feature) = &self.label_feature {
            label_feature.process_event(event).await?;
        }

        Ok(())
    }
}
