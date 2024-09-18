use super::model::{CommentId, IssueId, RepoId};

#[derive(Debug)]
pub struct GitEvent {
    pub repo_id: RepoId,
    pub issue_id: IssueId,
    pub kind: GitEventKind,
}

#[derive(Debug)]
pub enum GitEventKind {
    NewIssue,
    NewComment(CommentId),
}
