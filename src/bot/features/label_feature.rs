use std::str::FromStr;

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
use log::error;
use non_empty_string::NonEmptyString;
use serde::{Deserialize, Serialize};

use super::templates::{AuthorTemplate, IssueTemplate, LabelTemplate};

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
    pub labels: Vec<LabelTemplate>,
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
        if let GitEventKind::NewIssue = event.kind {
            let issue = self
                .githost
                .get_issue(event.repo_id, event.issue_id)
                .await?;

            let author = self.githost.get_user(issue.author_user_id).await?;

            let labels = self.githost.get_repo_labels(event.repo_id).await?;

            let context = LabelFeatureContext {
                issue: (issue, author).into(),
                labels: labels.into_iter().map(|l| l.into()).collect(),
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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use mockall::predicate;
    use non_empty_string::NonEmptyString;

    use crate::{
        bot::features::label_feature::LabelFeature,
        githost::{
            events::{GitEvent, GitEventKind},
            host::MockGitHost,
            model::{Issue, IssueId, RepoId, User, UserId},
        },
        llm::{
            agent::LlmAgent,
            llm_trait::{CompletionParameters, MockLlm},
            messages::AiMessage,
        },
    };

    const TEST_SYSTEM_MESSAGE: &str = "You are a bot that improves user issues on GitHub. When analyzing issues, please check that it mentions OS version and Rust version. If you think that all of this information is provided, then write special word EMPTY";
    const TEST_USER_MESSAGE: &str = "Here is the issue: {{ context.issue.body }}. Please write a comment to improve it or write EMPTY";

    #[tokio::test]
    async fn labels_issue() {
        let (git_event, issue, author) = make_test_data();

        let issue_clone = issue.clone();

        let llm_output: NonEmptyString = "bug, startup, needs refinement".try_into().unwrap();

        let mut githost_mock = MockGitHost::new();

        githost_mock
            .expect_get_issue()
            .with(
                predicate::eq(git_event.repo_id),
                predicate::eq(git_event.issue_id),
            )
            .returning(move |_, _| Ok(issue_clone.clone()));

        githost_mock
            .expect_get_repo_labels()
            .with(predicate::eq(git_event.repo_id))
            .returning(|_| Ok(vec![])); // Not really relevant. Labeling will hapen based on LLM
                                        // output, and because we don't use a real LLM, we provide our own correct reponse.

        for label in ["bug", "startup", "needs refinement"] {
            githost_mock
                .expect_assign_label()
                .with(
                    predicate::eq(git_event.repo_id),
                    predicate::eq(git_event.issue_id),
                    predicate::eq(NonEmptyString::from_str(label).unwrap()),
                )
                .times(1)
                .returning(|_, _, _| Ok(()));
        }

        githost_mock
            .expect_get_user()
            .with(predicate::eq(issue.author_user_id))
            .returning(move |_| Ok(author.clone()));

        let mut llm_mock = MockLlm::new();

        llm_mock
            .expect_complete()
            .returning(move |_, _, _| Ok(AiMessage::from(llm_output.clone())));

        let feature = LabelFeature::new(
            githost_mock,
            LlmAgent::build_raw(
                llm_mock,
                TEST_SYSTEM_MESSAGE.try_into().unwrap(),
                TEST_USER_MESSAGE.try_into().unwrap(),
                CompletionParameters::default(),
            )
            .unwrap(),
        );

        feature.process_event(&git_event).await.unwrap();
    }

    #[tokio::test]
    async fn doesnt_label_on_empty() {
        let (git_event, issue, author) = make_test_data();

        let issue_clone = issue.clone();

        let llm_output: NonEmptyString = "EMPTY".try_into().unwrap();

        let mut githost_mock = MockGitHost::new();

        githost_mock
            .expect_get_issue()
            .with(
                predicate::eq(git_event.repo_id),
                predicate::eq(git_event.issue_id),
            )
            .returning(move |_, _| Ok(issue_clone.clone()));

        githost_mock
            .expect_get_repo_labels()
            .with(predicate::eq(git_event.repo_id))
            .returning(|_| Ok(vec![]));

        githost_mock
            .expect_get_user()
            .with(predicate::eq(issue.author_user_id))
            .returning(move |_| Ok(author.clone()));

        let mut llm_mock = MockLlm::new();

        llm_mock
            .expect_complete()
            .returning(move |_, _, _| Ok(AiMessage::from(llm_output.clone())));

        let feature = LabelFeature::new(
            githost_mock,
            LlmAgent::build_raw(
                llm_mock,
                TEST_SYSTEM_MESSAGE.try_into().unwrap(),
                TEST_USER_MESSAGE.try_into().unwrap(),
                CompletionParameters::default(),
            )
            .unwrap(),
        );

        feature.process_event(&git_event).await.unwrap();
    }

    fn make_test_data() -> (GitEvent, Issue, User) {
        let repo_id = RepoId::from(1);
        let issue_id = IssueId::from(1 as usize);
        let user_id = UserId::from(1);

        let git_event = GitEvent {
            repo_id,
            issue_id,
            kind: GitEventKind::NewIssue,
        };

        let issue = Issue {
            id: issue_id,
            author_user_id: user_id,
            title: "Problem with your program".try_into().unwrap(),
            body: "Hi! I can't run your program".into(),
        };

        let author = User {
            id: user_id,
            nickname: "InAnYan".try_into().unwrap(),
        };

        (git_event, issue, author)
    }
}
