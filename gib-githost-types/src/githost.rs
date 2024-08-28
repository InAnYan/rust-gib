use async_trait::async_trait;
use non_empty_string::NonEmptyString;

use crate::{
    error::Result,
    event::GitEvent,
    gittypes::{CommentId, Issue, IssueId, Label, LabelId, Repo, RepoId, User, UserId},
};

#[async_trait]
pub trait GitHost {
    async fn poll_events(&mut self) -> Result<Vec<GitEvent>>;

    async fn get_user(&mut self, id: UserId) -> Result<User>;

    async fn get_repo(&mut self, id: RepoId) -> Result<Repo>;

    async fn get_issue(&mut self, repo_id: RepoId, issue_id: IssueId) -> Result<Issue>;

    async fn get_comment(
        &mut self,
        repo_id: RepoId,
        issue_id: IssueId,
        comment_id: CommentId,
    ) -> Result<Label>;

    async fn make_comment(
        &mut self,
        repo_id: RepoId,
        issue_id: IssueId,
        message: NonEmptyString,
    ) -> Result<Vec<Label>>;

    async fn get_repo_labels(&mut self, repo_id: RepoId) -> Result<Vec<Label>>;

    async fn assign_label(
        &mut self,
        repo_id: RepoId,
        issue_id: IssueId,
        label_id: LabelId,
    ) -> Result<()>;
}
