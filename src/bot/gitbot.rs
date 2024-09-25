use serde::Deserialize;
use tracing::instrument;

use crate::{
    githost::{events::GitEvent, host::GitHost},
    llm::llm::Llm,
};

use super::features::{
    improve_feature::{ImproveFeature, ImproveFeatureConfig, ImproveFeatureError},
    label_feature::{LabelFeature, LabelFeatureConfig, LabelFeatureError},
};

#[derive(Debug, thiserror::Error)]
pub enum GitBotError<GE, LE> {
    #[error("issue-improve feature returned an error")]
    ImproveFeatureError(#[from] ImproveFeatureError<GE, LE>),

    #[error("issue-label feature returned an error")]
    LabelFeatureError(#[from] LabelFeatureError<GE, LE>),
}

pub type Result<T, GE, LE> = std::result::Result<T, GitBotError<GE, LE>>;

#[derive(Deserialize)]
pub struct GitBotConfig {
    features: FeaturesConfig,
}

#[derive(Deserialize)]
pub struct FeaturesConfig {
    pub improve_feature: Option<ImproveFeatureConfig>,
    pub label_feature: Option<LabelFeatureConfig>,
}

pub struct GitBot<G, L> {
    improve_feature: Option<ImproveFeature<G, L>>,
    label_feature: Option<LabelFeature<G, L>>,
}

impl<G: GitHost + Clone, L: Llm + Clone> GitBot<G, L> {
    pub async fn build(
        config: GitBotConfig,
        githost: G,
        llm: L,
    ) -> Result<Self, G::Error, L::Error> {
        Ok(Self {
            improve_feature: match config.features.improve_feature {
                Some(config) => {
                    Some(ImproveFeature::build(config, githost.clone(), llm.clone()).await?)
                }
                None => None,
            },

            label_feature: match config.features.label_feature {
                Some(config) => {
                    Some(LabelFeature::build(config, githost.clone(), llm.clone()).await?)
                }
                None => None,
            },
        })
    }

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
