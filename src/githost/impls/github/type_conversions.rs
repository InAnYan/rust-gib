use crate::githost::gittypes::{CommentId, IssueId, RepoId};

impl From<octocrab::models::RepositoryId> for RepoId {
    fn from(value: octocrab::models::RepositoryId) -> Self {
        RepoId::from(*value as usize)
    }
}

impl From<octocrab::models::IssueId> for IssueId {
    fn from(value: octocrab::models::IssueId) -> Self {
        IssueId::from(*value as usize)
    }
}
impl From<octocrab::models::CommentId> for CommentId {
    fn from(value: octocrab::models::CommentId) -> Self {
        CommentId::from(*value as usize)
    }
}
