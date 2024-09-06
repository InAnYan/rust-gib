use async_trait::async_trait;
use non_empty_string::NonEmptyString;

use crate::{
    error::Result,
    gittypes::{CommentId, Issue, IssueId, Label, LabelId, Repo, RepoId, User, UserId},
};

#[async_trait]
pub trait GitHost {
    async fn get_user(&self, id: UserId) -> Result<User>;

    async fn get_repo(&self, id: RepoId) -> Result<Repo>;

    async fn get_issue(&self, repo_id: RepoId, issue_id: IssueId) -> Result<Issue>;

    async fn get_comment(
        &self,
        repo_id: RepoId,
        issue_id: IssueId,
        comment_id: CommentId,
    ) -> Result<Label>;

    async fn make_comment(
        &self,
        repo_id: RepoId,
        issue_id: IssueId,
        message: NonEmptyString,
    ) -> Result<Vec<Label>>;

    async fn get_repo_labels(&self, repo_id: RepoId) -> Result<Vec<Label>>;

    async fn assign_label(
        &self,
        repo_id: RepoId,
        issue_id: IssueId,
        label_id: LabelId,
    ) -> Result<()>;
}
