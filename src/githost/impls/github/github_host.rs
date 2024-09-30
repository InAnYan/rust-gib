use std::{path::PathBuf, str::FromStr};

use async_trait::async_trait;
use jsonwebtoken::EncodingKey;
use non_empty_string::NonEmptyString;
use octocrab::{Octocrab, OctocrabBuilder};
use secrecy::{ExposeSecret, SecretVec};
use serde::Deserialize;
use tokio::fs::read;
use url::Url;

use crate::{
    githost::{
        host::GitHost,
        model::{Comment, CommentId, Issue, IssueId, Label, LabelId, Repo, RepoId, User, UserId},
    },
    utils::clear_url::clear_url,
};

use super::errors::GithubError;

#[derive(Deserialize)]
pub struct GithubConfig {
    pub bot_name: NonEmptyString,
    pub app_id: u64,
    pub installation_id: u64,
    pub pem_rsa_key_path: PathBuf,
}

#[derive(Clone)]
pub struct GithubHost {
    octocrab: Octocrab,
    bot_name: NonEmptyString,
}

const GITHUB_API_URL: &str = "https://api.github.com";

impl GithubHost {
    pub async fn build(config: GithubConfig) -> Result<Self, GithubError> {
        Self::build_raw(
            config.bot_name,
            config.app_id,
            config.installation_id,
            SecretVec::new(
                read(config.pem_rsa_key_path)
                    .await
                    .map_err(GithubError::SecretKeyFileOpenError)?,
            ),
            Url::from_str(GITHUB_API_URL).unwrap(),
        )
        .await
    }

    pub async fn build_raw(
        bot_name: NonEmptyString,
        app_id: u64,
        installation_id: u64,
        pem_rsa_key: SecretVec<u8>,
        api_url: Url,
    ) -> Result<Self, GithubError> {
        let octocrab = OctocrabBuilder::new()
            .app(
                app_id.into(),
                EncodingKey::from_rsa_pem(pem_rsa_key.expose_secret().as_slice())?,
            )
            .base_uri(clear_url(api_url))?
            .build()?;

        let octocrab = octocrab.installation(installation_id.into());

        Ok(Self { octocrab, bot_name })
    }
}

#[async_trait]
impl GitHost for GithubHost {
    type Error = GithubError;

    fn get_self_name(&self) -> &NonEmptyString {
        &self.bot_name
    }

    async fn get_user(&self, id: UserId) -> Result<User, Self::Error> {
        let profile = self
            .octocrab
            .users_by_id(octocrab::models::UserId::from(*id as u64))
            .profile()
            .await?;

        Ok(User {
            id,
            nickname: profile
                .login
                .try_into()
                .map_err(|_| GithubError::ApiResponseInvalidFormatError)?,
        })
    }

    async fn get_repo(&self, id: RepoId) -> Result<Repo, Self::Error> {
        let repo = self
            .octocrab
            .repos_by_id(octocrab::models::RepositoryId::from(*id as u64))
            .get()
            .await?;

        Ok(Repo {
            id,
            owner: repo
                .owner
                .ok_or(GithubError::ApiResponseInvalidFormatError)?
                .login,
            name: repo.name,
        })
    }

    async fn get_issue(&self, repo_id: RepoId, issue_id: IssueId) -> Result<Issue, Self::Error> {
        let issue = self
            .octocrab
            .issues_by_id(octocrab::models::RepositoryId::from(*repo_id as u64))
            .get(*issue_id as u64)
            .await?;

        Ok(Issue {
            id: issue_id,
            title: issue
                .title
                .try_into()
                .map_err(|_| GithubError::ApiResponseInvalidFormatError)?,
            body: issue.body.unwrap_or(String::new()),
            author_user_id: issue.user.id.into(),
        })
    }

    async fn get_comment(
        &self,
        repo_id: RepoId,
        _issue_id: IssueId,
        comment_id: CommentId,
    ) -> Result<Comment, Self::Error> {
        let comment = self
            .octocrab
            .issues_by_id(octocrab::models::RepositoryId::from(*repo_id as u64))
            .get_comment(octocrab::models::CommentId::from(*comment_id as u64))
            .await?;

        Ok(Comment {
            id: comment_id,
            user_id: UserId::from(*comment.user.id as usize),
            body: comment
                .body
                .ok_or(GithubError::ApiResponseInvalidFormatError)?
                .try_into()
                .map_err(|_| GithubError::ApiResponseInvalidFormatError)?,
        })
    }

    async fn make_comment(
        &self,
        repo_id: RepoId,
        issue_id: IssueId,
        message: NonEmptyString,
    ) -> Result<(), Self::Error> {
        self.octocrab
            .issues_by_id(octocrab::models::RepositoryId::from(*repo_id as u64))
            .create_comment(*issue_id as u64, message)
            .await?;

        Ok(())
    }

    async fn get_repo_labels(&self, repo_id: RepoId) -> Result<Vec<Label>, Self::Error> {
        let labels_numbers = self
            .octocrab
            .issues_by_id(octocrab::models::RepositoryId::from(*repo_id as u64))
            .list_labels_for_repo()
            .send()
            .await?;

        let mut labels = Vec::new();

        for i in 0..(labels_numbers.number_of_pages().unwrap_or(1)) {
            let labels_page = self
                .octocrab
                .issues_by_id(octocrab::models::RepositoryId::from(*repo_id as u64))
                .list_labels_for_repo()
                .page(i)
                .send()
                .await?;

            for label in labels_page.into_iter() {
                labels.push(Label {
                    id: LabelId::from(*label.id as usize),
                    name: label
                        .name
                        .try_into()
                        .map_err(|_| GithubError::ApiResponseInvalidFormatError)?,
                    description: label.description.unwrap_or("".into()),
                });
            }
        }

        Ok(labels)
    }

    async fn assign_label(
        &self,
        repo_id: RepoId,
        issue_id: IssueId,
        label_name: NonEmptyString,
    ) -> Result<(), Self::Error> {
        self.octocrab
            .issues_by_id(octocrab::models::RepositoryId::from(*repo_id as u64))
            .add_labels(*issue_id as u64, &[label_name.into()])
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use secrecy::SecretVec;
    use serde_json::json;
    use url::Url;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::githost::{
        host::GitHost,
        impls::github::github_host::GithubHost,
        model::{CommentId, IssueId, Label, LabelId, RepoId, UserId},
    };

    async fn setup() -> (MockServer, GithubHost) {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/app/installations/1/access_tokens"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(include_str!("access_token.json")),
            )
            .mount(&mock_server)
            .await;

        let github = GithubHost::build_raw(
            "bot".try_into().unwrap(),
            1,
            1,
            SecretVec::new(include_bytes!("test.pem").into()),
            Url::from_str(&mock_server.uri().to_string()).unwrap(),
        )
        .await
        .unwrap();

        (mock_server, github)
    }

    #[tokio::test]
    async fn get_user() {
        let (mock_server, github) = setup().await;

        Mock::given(method("GET"))
            .and(path("/user/1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
              "login": "octocat",
              "id": 1,
              "node_id": "MDQ6VXNlcjE=",
              "avatar_url": "https://github.com/images/error/octocat_happy.gif",
              "gravatar_id": "",
              "url": "https://api.github.com/users/octocat",
              "html_url": "https://github.com/octocat",
              "followers_url": "https://api.github.com/users/octocat/followers",
              "following_url": "https://api.github.com/users/octocat/following{/other_user}",
              "gists_url": "https://api.github.com/users/octocat/gists{/gist_id}",
              "starred_url": "https://api.github.com/users/octocat/starred{/owner}{/repo}",
              "subscriptions_url": "https://api.github.com/users/octocat/subscriptions",
              "organizations_url": "https://api.github.com/users/octocat/orgs",
              "repos_url": "https://api.github.com/users/octocat/repos",
              "events_url": "https://api.github.com/users/octocat/events{/privacy}",
              "received_events_url": "https://api.github.com/users/octocat/received_events",
              "type": "User",
              "site_admin": false,
              "name": "monalisa octocat",
              "company": "GitHub",
              "blog": "https://github.com/blog",
              "location": "San Francisco",
              "email": "octocat@github.com",
              "hireable": false,
              "bio": "There once was...",
              "twitter_username": "monatheoctocat",
              "public_repos": 2,
              "public_gists": 1,
              "followers": 20,
              "following": 0,
              "created_at": "2008-01-14T04:33:35Z",
              "updated_at": "2008-01-14T04:33:35Z"
            })))
            .mount(&mock_server)
            .await;

        let user = github.get_user(UserId::from(1)).await.unwrap();

        assert_eq!(user.id, UserId::from(1));
        assert_eq!(user.nickname.as_str(), "octocat");
    }

    #[tokio::test]
    async fn get_issue() {
        let (mock_server, github) = setup().await;

        Mock::given(method("GET"))
            .and(path("/repositories/1/issues/1"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(include_str!("issue_response.json")),
            )
            .mount(&mock_server)
            .await;

        let issue = github
            .get_issue(RepoId::from(1), IssueId::from(1 as usize))
            .await
            .unwrap();

        assert_eq!(issue.id, IssueId::from(1 as usize));
        assert_eq!(issue.title.as_str(), "Found a bug");
        assert_eq!(issue.body.as_str(), "I'm having a problem with this.");
    }

    #[tokio::test]
    async fn get_repo() {
        let (mock_server, github) = setup().await;

        Mock::given(method("GET"))
            .and(path("/repositories/1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(include_str!("repository_response.json")),
            )
            .mount(&mock_server)
            .await;

        let repo = github.get_repo(RepoId::from(1)).await.unwrap();

        assert_eq!(repo.id, RepoId::from(1));
        assert_eq!(repo.owner.as_str(), "octocat");
        assert_eq!(repo.name.as_str(), "Hello-World");
    }

    #[tokio::test]
    async fn get_comment() {
        let (mock_server, github) = setup().await;

        Mock::given(method("GET"))
            .and(path("/repositories/1/issues/comments/1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
              "id": 1,
              "node_id": "MDEyOklzc3VlQ29tbWVudDE=",
              "url": "https://api.github.com/repos/octocat/Hello-World/issues/comments/1",
              "html_url": "https://github.com/octocat/Hello-World/issues/1347#issuecomment-1",
              "body": "Me too",
              "user": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "avatar_url": "https://github.com/images/error/octocat_happy.gif",
                "gravatar_id": "",
                "url": "https://api.github.com/users/octocat",
                "html_url": "https://github.com/octocat",
                "followers_url": "https://api.github.com/users/octocat/followers",
                "following_url": "https://api.github.com/users/octocat/following{/other_user}",
                "gists_url": "https://api.github.com/users/octocat/gists{/gist_id}",
                "starred_url": "https://api.github.com/users/octocat/starred{/owner}{/repo}",
                "subscriptions_url": "https://api.github.com/users/octocat/subscriptions",
                "organizations_url": "https://api.github.com/users/octocat/orgs",
                "repos_url": "https://api.github.com/users/octocat/repos",
                "events_url": "https://api.github.com/users/octocat/events{/privacy}",
                "received_events_url": "https://api.github.com/users/octocat/received_events",
                "type": "User",
                "site_admin": false
              },
              "created_at": "2011-04-14T16:00:49Z",
              "updated_at": "2011-04-14T16:00:49Z",
              "issue_url": "https://api.github.com/repos/octocat/Hello-World/issues/1347",
              "author_association": "COLLABORATOR"
            })))
            .mount(&mock_server)
            .await;

        let comment = github
            .get_comment(
                RepoId::from(1),
                IssueId::from(1 as usize),
                CommentId::from(1),
            )
            .await
            .unwrap();

        assert_eq!(comment.id, CommentId::from(1));
        assert_eq!(comment.user_id, UserId::from(1));
        assert_eq!(comment.body.as_str(), "Me too");
    }

    #[tokio::test]
    async fn make_comment() {
        let (mock_server, github) = setup().await;

        Mock::given(method("POST"))
            .and(path("/repositories/1/issues/1/comments"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
              "id": 1,
              "node_id": "MDEyOklzc3VlQ29tbWVudDE=",
              "url": "https://api.github.com/repos/octocat/Hello-World/issues/comments/1",
              "html_url": "https://github.com/octocat/Hello-World/issues/1347#issuecomment-1",
              "body": "Me too",
              "user": {
                "login": "octocat",
                "id": 1,
                "node_id": "MDQ6VXNlcjE=",
                "avatar_url": "https://github.com/images/error/octocat_happy.gif",
                "gravatar_id": "",
                "url": "https://api.github.com/users/octocat",
                "html_url": "https://github.com/octocat",
                "followers_url": "https://api.github.com/users/octocat/followers",
                "following_url": "https://api.github.com/users/octocat/following{/other_user}",
                "gists_url": "https://api.github.com/users/octocat/gists{/gist_id}",
                "starred_url": "https://api.github.com/users/octocat/starred{/owner}{/repo}",
                "subscriptions_url": "https://api.github.com/users/octocat/subscriptions",
                "organizations_url": "https://api.github.com/users/octocat/orgs",
                "repos_url": "https://api.github.com/users/octocat/repos",
                "events_url": "https://api.github.com/users/octocat/events{/privacy}",
                "received_events_url": "https://api.github.com/users/octocat/received_events",
                "type": "User",
                "site_admin": false
              },
              "created_at": "2011-04-14T16:00:49Z",
              "updated_at": "2011-04-14T16:00:49Z",
              "issue_url": "https://api.github.com/repos/octocat/Hello-World/issues/1347",
              "author_association": "COLLABORATOR"
            })))
            .mount(&mock_server)
            .await;

        github
            .make_comment(
                RepoId::from(1),
                IssueId::from(1 as usize),
                "message".try_into().unwrap(),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn get_repo_labels() {
        let (mock_server, github) = setup().await;

        Mock::given(method("GET"))
            .and(path("/repositories/1/labels"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
              {
                "id": 1,
                "node_id": "MDU6TGFiZWwyMDgwNDU5NDY=",
                "url": "https://api.github.com/repos/octocat/Hello-World/labels/bug",
                "name": "bug",
                "description": "Something isn't working",
                "color": "f29513",
                "default": true
              },
              {
                "id": 2,
                "node_id": "MDU6TGFiZWwyMDgwNDU5NDc=",
                "url": "https://api.github.com/repos/octocat/Hello-World/labels/enhancement",
                "name": "enhancement",
                "description": "New feature or request",
                "color": "a2eeef",
                "default": false
              }
            ])))
            .mount(&mock_server)
            .await;

        let labels = github.get_repo_labels(RepoId::from(1)).await.unwrap();

        assert_eq!(
            labels,
            vec![
                Label {
                    id: LabelId::from(1),
                    name: "bug".try_into().unwrap(),
                    description: "Something isn't working".to_owned()
                },
                Label {
                    id: LabelId::from(2),
                    name: "enhancement".try_into().unwrap(),
                    description: "New feature or request".to_owned()
                }
            ]
        )
    }

    #[tokio::test]
    async fn assign_label() {
        let (mock_server, github) = setup().await;

        Mock::given(method("POST"))
            .and(path("/repositories/1/issues/1/labels"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .mount(&mock_server)
            .await;

        github
            .assign_label(
                RepoId::from(1),
                IssueId::from(1 as usize),
                "bug".try_into().unwrap(),
            )
            .await
            .unwrap();
    }
}
