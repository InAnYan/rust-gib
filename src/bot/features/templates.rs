use non_empty_string::NonEmptyString;
use serde::Serialize;

use crate::githost::model::{Issue, IssueId, Label, User};

#[derive(Serialize, Debug)]
pub struct IssueTemplate {
    pub number: IssueId,
    pub author: AuthorTemplate,
    pub title: NonEmptyString,
    pub body: String, // Can be empty.
}

impl From<(Issue, User)> for IssueTemplate {
    fn from((issue, user): (Issue, User)) -> Self {
        IssueTemplate {
            number: issue.id,
            author: user.into(),
            title: issue.title,
            body: issue.body,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct AuthorTemplate {
    pub nickname: NonEmptyString,
}

impl From<User> for AuthorTemplate {
    fn from(value: User) -> Self {
        AuthorTemplate {
            nickname: value.nickname,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct LabelTemplate {
    pub name: NonEmptyString,
    pub description: String, // Can be empty.
}

impl From<Label> for LabelTemplate {
    fn from(value: Label) -> Self {
        LabelTemplate {
            name: value.name,
            description: value.description,
        }
    }
}
