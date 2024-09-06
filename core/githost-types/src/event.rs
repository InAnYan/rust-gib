use crate::gittypes::{CommentId, IssueId, RepoId};

pub struct GitEvent {
    pub repo_id: RepoId,
    pub issue_id: IssueId,
    pub kind: GitEventKind,
}

pub enum GitEventKind {
    NewIssue,
    NewComment(CommentId),
}
