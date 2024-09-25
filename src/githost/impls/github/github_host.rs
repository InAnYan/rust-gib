use std::path::PathBuf;

use async_trait::async_trait;
use jsonwebtoken::EncodingKey;
use non_empty_string::NonEmptyString;
use octocrab::{Octocrab, OctocrabBuilder};
use serde::Deserialize;
use tokio::fs::read;

use crate::githost::{
    host::GitHost,
    model::{Comment, CommentId, Issue, IssueId, Label, LabelId, Repo, RepoId, User, UserId},
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

impl GithubHost {
    pub async fn build(config: GithubConfig) -> Result<Self, GithubError> {
        let octocrab = OctocrabBuilder::new()
            .app(
                config.app_id.into(),
                EncodingKey::from_rsa_pem(
                    read(config.pem_rsa_key_path)
                        .await
                        .map_err(GithubError::SecretKeyFileOpenError)?
                        .as_slice(),
                )?,
            )
            .build()?;

        let octocrab = octocrab.installation(config.installation_id.into());

        Ok(Self {
            octocrab,
            bot_name: config.bot_name,
        })
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
