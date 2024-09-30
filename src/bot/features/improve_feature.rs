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

#[cfg(test)]
mod tests {
    use mockall::predicate;
    use non_empty_string::NonEmptyString;

    use crate::{
        bot::features::improve_feature::ImproveFeature,
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
    async fn improves_bad_issue() {
        let (git_event, issue, author) = make_test_data();

        let issue_clone = issue.clone();

        let llm_output: NonEmptyString =
            "Please state your OS and Rust version".try_into().unwrap();

        let mut githost_mock = MockGitHost::new();

        githost_mock
            .expect_get_issue()
            .with(
                predicate::eq(git_event.repo_id),
                predicate::eq(git_event.issue_id),
            )
            .returning(move |_, _| Ok(issue_clone.clone()));

        githost_mock
            .expect_make_comment()
            .with(
                predicate::eq(git_event.repo_id),
                predicate::eq(git_event.issue_id),
                predicate::eq(llm_output.clone()),
            )
            .returning(|_, _, _| Ok(()));

        githost_mock
            .expect_get_user()
            .with(predicate::eq(issue.author_user_id))
            .returning(move |_| Ok(author.clone()));

        let mut llm_mock = MockLlm::new();

        llm_mock
            .expect_complete()
            .returning(move |_, _, _| Ok(AiMessage::from(llm_output.clone())));

        let feature = ImproveFeature::new(
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
    async fn doesnt_improve_on_empty() {
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
            .expect_get_user()
            .with(predicate::eq(issue.author_user_id))
            .returning(move |_| Ok(author.clone()));

        let mut llm_mock = MockLlm::new();

        llm_mock
            .expect_complete()
            .returning(move |_, _, _| Ok(AiMessage::from(llm_output.clone())));

        let feature = ImproveFeature::new(
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
