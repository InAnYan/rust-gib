use std::str::FromStr;

use crate::{
    githost::{
        events::{GitEvent, GitEventKind},
        host::GitHost,
    },
    llm::{
        agent::{LlmAgent, LlmAgentConfig, LlmAgentError},
        llm::Llm,
    },
};
use log::error;
use non_empty_string::NonEmptyString;
use serde::{Deserialize, Serialize};

use super::templates::{AuthorTemplate, IssueTemplate};

#[derive(Debug, thiserror::Error)]
pub enum LabelFeatureError<GE, LE> {
    #[error("error from LLM agent")]
    LlmAgentError(#[source] LlmAgentError<LE>),

    #[error("unable to perform Git host action")]
    GitHostError(#[from] GE),
}

pub type Result<T, GE, LE> = std::result::Result<T, LabelFeatureError<GE, LE>>;

#[derive(Deserialize)]
pub struct LabelFeatureConfig {
    agent: LlmAgentConfig,
}

pub struct LabelFeature<G, L> {
    githost: G,
    agent: LlmAgent<L, LabelFeatureContext>,
}

#[derive(Serialize, Debug)]
pub struct LabelFeatureContext {
    pub issue: IssueTemplate,
}

impl<G: GitHost, L: Llm> LabelFeature<G, L> {
    pub async fn build_from_config(
        config: LabelFeatureConfig,
        githost: G,
        llm: L,
    ) -> Result<Self, G::Error, L::Error> {
        let agent = LlmAgent::build_from_config(llm, config.agent)
            .map_err(LabelFeatureError::LlmAgentError)?;

        Ok(Self::new(githost, agent))
    }

    pub fn new(githost: G, agent: LlmAgent<L, LabelFeatureContext>) -> Self {
        Self { githost, agent }
    }

    pub async fn process_event(&self, event: &GitEvent) -> Result<(), G::Error, L::Error> {
        match event.kind {
            GitEventKind::NewIssue => {
                let issue = self
                    .githost
                    .get_issue(event.repo_id, event.issue_id)
                    .await?;

                let author = self.githost.get_user(issue.author_user_id).await?;

                let context = LabelFeatureContext {
                    issue: IssueTemplate {
                        number: event.issue_id,
                        author: AuthorTemplate {
                            nickname: author.nickname,
                        },

                        title: issue.title,
                        body: issue.body,
                    },
                };

                let ai_message = self
                    .agent
                    .process(&context)
                    .await
                    .map_err(LabelFeatureError::LlmAgentError)?;

                if !ai_message.as_str().starts_with("EMPTY") {
                    for label in ai_message.as_str().split(", ") {
                        match NonEmptyString::from_str(label) {
                            Err(e) => {
                                error!("AI has generated malformed result: {:?}. Skipping.", e);
                            }

                            Ok(label) => {
                                self.githost
                                    .assign_label(event.repo_id, event.issue_id, label)
                                    .await?;
                            }
                        }
                    }
                }
            }

            _ => {}
        }

        Ok(())
    }
}
