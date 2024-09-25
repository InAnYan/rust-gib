use async_trait::async_trait;
use non_empty_string::NonEmptyString;

use super::model::{Comment, CommentId, Issue, IssueId, Label, Repo, RepoId, User, UserId};

#[async_trait]
pub trait GitHost {
    type Error;

    fn get_self_name(&self) -> &NonEmptyString;

    async fn get_user(&self, id: UserId) -> Result<User, Self::Error>;

    async fn get_repo(&self, id: RepoId) -> Result<Repo, Self::Error>;

    async fn get_issue(&self, repo_id: RepoId, issue_id: IssueId) -> Result<Issue, Self::Error>;

    async fn get_comment(
        &self,
        repo_id: RepoId,
        issue_id: IssueId,
        comment_id: CommentId,
    ) -> Result<Comment, Self::Error>;

    async fn make_comment(
        &self,
        repo_id: RepoId,
        issue_id: IssueId,
        message: NonEmptyString,
    ) -> Result<(), Self::Error>;

    async fn get_repo_labels(&self, repo_id: RepoId) -> Result<Vec<Label>, Self::Error>;

    // NOTE: It seems GitHub does not support getting information about label through label id. So
    // I left this API like this (using label name).
    async fn assign_label(
        &self,
        repo_id: RepoId,
        issue_id: IssueId,
        label_name: NonEmptyString,
    ) -> Result<(), Self::Error>;
}
