use crate::{
    githost::{
        events::{GitEvent, GitEventKind},
        host::GitHost,
    },
    llm::{
        agent::{LlmAgent, LlmAgentConfig, LlmAgentError},
        llm_trait::Llm,
    },
};
use serde::{Deserialize, Serialize};

use super::templates::{AuthorTemplate, IssueTemplate};

#[derive(Debug, thiserror::Error)]
pub enum ImproveFeatureError<GE, LE> {
    #[error("error from LLM agent")]
    LlmAgentError(#[source] LlmAgentError<LE>),

    #[error("unable to perform Git host action")]
    GitHostError(#[from] GE),
}

pub type Result<T, GE, LE> = std::result::Result<T, ImproveFeatureError<GE, LE>>;

#[derive(Deserialize)]
pub struct ImproveFeatureConfig {
    agent: LlmAgentConfig,
}

pub struct ImproveFeature<G, L> {
    githost: G,
    agent: LlmAgent<L, ImproveFeatureContext>,
}

#[derive(Serialize, Debug)]
pub struct ImproveFeatureContext {
    pub issue: IssueTemplate,
}

impl<G: GitHost, L: Llm> ImproveFeature<G, L> {
    pub async fn build_from_config(
        config: ImproveFeatureConfig,
        githost: G,
        llm: L,
    ) -> Result<Self, G::Error, L::Error> {
        let agent = LlmAgent::build_from_config(llm, config.agent)
            .map_err(ImproveFeatureError::LlmAgentError)?;

        Ok(Self::new(githost, agent))
    }

    pub fn new(githost: G, agent: LlmAgent<L, ImproveFeatureContext>) -> Self {
        Self { githost, agent }
    }

    pub async fn process_event(&self, event: &GitEvent) -> Result<(), G::Error, L::Error> {
        if let GitEventKind::NewIssue = event.kind {
            let issue = self
                .githost
                .get_issue(event.repo_id, event.issue_id)
                .await?;

            let author = self.githost.get_user(issue.author_user_id).await?;

            let context = ImproveFeatureContext {
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
                .map_err(ImproveFeatureError::LlmAgentError)?;

            if !ai_message.as_str().starts_with("EMPTY") {
                self.githost
                    .make_comment(event.repo_id, event.issue_id, ai_message)
                    .await?;
            }
        }

        Ok(())
    }
}
