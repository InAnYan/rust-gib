use async_trait::async_trait;
use gib_githost_types::{
    error::Result,
    event::GitEvent,
    githost::GitHost,
    gittypes::{CommentId, Issue, IssueId, Label, LabelId, Repo, RepoId, User, UserId},
};
use non_empty_string::NonEmptyString;

pub struct GitHubHost {}

#[async_trait]
impl GitHost for GitHubHost {
    async fn poll_events(&mut self) -> Result<Vec<GitEvent>> {
        todo!()
    }

    async fn get_user(&mut self, id: UserId) -> Result<User> {
        todo!()
    }

    async fn get_repo(&mut self, id: RepoId) -> Result<Repo> {
        todo!()
    }

    async fn get_issue(&mut self, repo_id: RepoId, issue_id: IssueId) -> Result<Issue> {
        todo!()
    }

    async fn get_comment(
        &mut self,
        repo_id: RepoId,
        issue_id: IssueId,
        comment_id: CommentId,
    ) -> Result<Label> {
        todo!()
    }

    async fn make_comment(
        &mut self,
        repo_id: RepoId,
        issue_id: IssueId,
        message: NonEmptyString,
    ) -> Result<Vec<Label>> {
        todo!()
    }

    async fn get_repo_labels(&mut self, repo_id: RepoId) -> Result<Vec<Label>> {
        todo!()
    }

    async fn assign_label(
        &mut self,
        repo_id: RepoId,
        issue_id: IssueId,
        label_id: LabelId,
    ) -> Result<()> {
        todo!()
    }
}
