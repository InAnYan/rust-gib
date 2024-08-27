use non_empty_string::NonEmptyString;

use crate::gittypes::{IssueId, Repo};

pub struct GitAction {
    pub kind: GitActionKind,
    // See GitEvent for reason why GitAction is struct and not a enum.
}

pub enum GitActionKind {
    SendMessageToIssueDiscussion {
        repo: Repo,
        id: IssueId,
        message: NonEmptyString,
    },
}
