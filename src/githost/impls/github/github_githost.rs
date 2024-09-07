use async_trait::async_trait;
use jsonwebtoken::EncodingKey;
use non_empty_string::NonEmptyString;
use octocrab::{Octocrab, OctocrabBuilder};
use secrecy::{ExposeSecret, SecretVec};

use crate::{
    errors::{GibError, Result},
    githost::{
        githost::GitHost,
        gittypes::{
            Comment, CommentId, Issue, IssueId, Label, LabelId, Repo, RepoId, User, UserId,
        },
    },
};

pub struct GitHubHost {
    octocrab: Octocrab,
}

impl GitHubHost {
    pub fn build(app_id: u64, key_pem_rsa: SecretVec<u8>) -> Result<Self> {
        let octocrab = OctocrabBuilder::new()
            .app(
                app_id.into(),
                EncodingKey::from_rsa_pem(key_pem_rsa.expose_secret().as_slice())
                    .map_err(|_| GibError::SecretKeyDecodeError)?,
            )
            .build()
            .map_err(|_| GibError::UnknownError)?;

        Ok(Self { octocrab })
    }
}

#[async_trait]
impl GitHost for GitHubHost {
    async fn get_user(&self, id: UserId) -> Result<User> {
        let profile = self
            .octocrab
            .users_by_id(octocrab::models::UserId::from(*id as u64))
            .profile()
            .await
            .map_err(|e| GibError::GitHostRequestError)?;

        Ok(User {
            id,
            nickname: profile
                .login
                .try_into()
                .map_err(|_| GibError::ApiResponseInvalidFormatError)?,
        })
    }

    async fn get_repo(&self, id: RepoId) -> Result<Repo> {
        let repo = self
            .octocrab
            .repos_by_id(octocrab::models::RepositoryId::from(*id as u64))
            .get()
            .await
            .map_err(|e| GibError::GitHostRequestError)?;

        Ok(Repo {
            id,
            owner: repo
                .owner
                .ok_or(GibError::ApiResponseInvalidFormatError)?
                .login,
            name: repo.name,
        })
    }

    async fn get_issue(&self, repo_id: RepoId, issue_id: IssueId) -> Result<Issue> {
        let issue = self
            .octocrab
            .issues_by_id(octocrab::models::RepositoryId::from(*repo_id as u64))
            .get(*issue_id as u64) // TODO: Insonsistency in the library. Expected IssueId or issue
            // number. Ahh, that's probably why there is no custom type.
            .await
            .map_err(|e| GibError::GitHostRequestError)?;

        Ok(Issue {
            id: issue_id,
            body: issue
                .body
                .ok_or(GibError::ApiResponseInvalidFormatError)?
                .try_into()
                .map_err(|_| GibError::ApiResponseInvalidFormatError)?,
        })
    }

    async fn get_comment(
        &self,
        repo_id: RepoId,
        _issue_id: IssueId,
        comment_id: CommentId,
    ) -> Result<Comment> {
        let comment = self
            .octocrab
            .issues_by_id(octocrab::models::RepositoryId::from(*repo_id as u64))
            .get_comment(octocrab::models::CommentId::from(*comment_id as u64))
            .await
            .map_err(|e| GibError::GitHostRequestError)?;

        Ok(Comment {
            id: comment_id,
            user_id: UserId::from(*comment.user.id as usize),
            body: comment
                .body
                .ok_or(GibError::ApiResponseInvalidFormatError)?
                .try_into()
                .map_err(|_| GibError::ApiResponseInvalidFormatError)?,
        })
    }

    async fn make_comment(
        &self,
        repo_id: RepoId,
        issue_id: IssueId,
        message: NonEmptyString,
    ) -> Result<()> {
        self.octocrab
            .issues_by_id(octocrab::models::RepositoryId::from(*repo_id as u64))
            .create_comment(*issue_id as u64, message)
            .await
            .map_err(|e| GibError::GitHostRequestError)?;

        Ok(())
    }

    async fn get_repo_labels(&self, repo_id: RepoId) -> Result<Vec<Label>> {
        let labels_numbers = self
            .octocrab
            .issues_by_id(octocrab::models::RepositoryId::from(*repo_id as u64))
            .list_labels_for_repo()
            .send()
            .await
            .map_err(|e| GibError::GitHostRequestError)?;

        let mut labels = Vec::new();

        for i in 0..(labels_numbers.number_of_pages().unwrap_or(1)) {
            let labels_page = self
                .octocrab
                .issues_by_id(octocrab::models::RepositoryId::from(*repo_id as u64))
                .list_labels_for_repo()
                .page(i)
                .send()
                .await
                .map_err(|e| GibError::GitHostRequestError)?;

            for label in labels_page.into_iter() {
                labels.push(Label {
                    id: LabelId::from(*label.id as usize),
                    name: label
                        .name
                        .try_into()
                        .map_err(|_| GibError::ApiResponseInvalidFormatError)?,
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
        label_id: LabelId,
    ) -> Result<()> {
        let labels = self.get_repo_labels(repo_id).await?;

        // TODO: It seems GitHub does not support getting information about label through label id.

        let label = labels
            .into_iter()
            .find(|l| l.id == label_id)
            .ok_or(GibError::ApiResponseInvalidFormatError)?;

        self.octocrab
            .issues_by_id(octocrab::models::RepositoryId::from(*repo_id as u64))
            .add_labels(*issue_id as u64, &[label.name.into()])
            .await
            .map_err(|e| GibError::GitHostRequestError)?;

        Ok(())
    }
}