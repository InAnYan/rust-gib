use non_empty_string::NonEmptyString;
use serde::Serialize;

use crate::githost::model::IssueId;

#[derive(Serialize, Debug)]
pub struct IssueTemplate {
    pub number: IssueId,
    pub author: AuthorTemplate,
    pub title: NonEmptyString,
    pub body: String, // Can be empty.
}

#[derive(Serialize, Debug)]
pub struct AuthorTemplate {
    pub nickname: NonEmptyString,
}

#[derive(Serialize, Debug)]
pub struct LabelTemplate {
    pub name: NonEmptyString,
    pub description: String, // Can be empty.
}
