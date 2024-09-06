use async_trait::async_trait;
use githost_types::{
    error::Result,
    event::GitEvent,
    githost::GitHost,
    gittypes::{CommentId, Issue, IssueId, Label, LabelId, Repo, RepoId, User, UserId},
};
use non_empty_string::NonEmptyString;

pub struct GitHubHost {}

#[async_trait]
impl GitHost for GitHubHost {
    async fn get_user(&self, id: UserId) -> Result<User> {
        todo!()
    }

    async fn get_repo(&self, id: RepoId) -> Result<Repo> {
        todo!()
    }

    async fn get_issue(&self, repo_id: RepoId, issue_id: IssueId) -> Result<Issue> {
        todo!()
    }

    async fn get_comment(
        &self,
        repo_id: RepoId,
        issue_id: IssueId,
        comment_id: CommentId,
    ) -> Result<Label> {
        todo!()
    }

    async fn make_comment(
        &self,
        repo_id: RepoId,
        issue_id: IssueId,
        message: NonEmptyString,
    ) -> Result<Vec<Label>> {
        todo!()
    }

    async fn get_repo_labels(&self, repo_id: RepoId) -> Result<Vec<Label>> {
        todo!()
    }

    async fn assign_label(
        &self,
        repo_id: RepoId,
        issue_id: IssueId,
        label_id: LabelId,
    ) -> Result<()> {
        todo!()
    }
}
