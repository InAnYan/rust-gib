use crate::githost::gittypes::{CommentId, IssueId, RepoId};

impl From<octocrab::models::RepositoryId> for RepoId {
    fn from(value: octocrab::models::RepositoryId) -> Self {
        RepoId::from(*value as usize)
    }
}

impl From<u64> for IssueId {
    fn from(value: u64) -> Self {
        IssueId::from(value as usize)
    }
}
impl From<octocrab::models::CommentId> for CommentId {
    fn from(value: octocrab::models::CommentId) -> Self {
        CommentId::from(*value as usize)
    }
}
