use super::model::{CommentId, IssueId, RepoId};

#[derive(Debug, PartialEq)]
pub struct GitEvent {
    pub repo_id: RepoId,
    pub issue_id: IssueId,
    pub kind: GitEventKind,
}

#[derive(Debug, PartialEq)]
pub enum GitEventKind {
    NewIssue,
    NewComment(CommentId),
}
