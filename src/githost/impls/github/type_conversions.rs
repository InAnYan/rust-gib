use crate::githost::model::{CommentId, IssueId, RepoId, UserId};

impl From<octocrab::models::UserId> for UserId {
    fn from(value: octocrab::models::UserId) -> Self {
        UserId::from(*value as usize)
    }
}

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
