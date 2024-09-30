use derive_more::derive::{AsRef, Deref, From};
use non_empty_string::NonEmptyString;
use serde::Serialize;

#[derive(Clone, Copy, From, AsRef, Deref, PartialEq, Debug)]
pub struct UserId(usize);

#[derive(Clone)]
pub struct User {
    pub id: UserId,
    pub nickname: NonEmptyString,
}

#[derive(Clone, Copy, From, AsRef, Deref, Debug, PartialEq)]
pub struct RepoId(usize);

pub struct Repo {
    pub id: RepoId,
    pub owner: String,
    pub name: String,
}

#[derive(Serialize, Clone, Copy, From, AsRef, Deref, Debug, PartialEq)]
#[serde(transparent)]
pub struct IssueId(usize);

#[derive(Clone)]
pub struct Issue {
    pub id: IssueId,
    pub author_user_id: UserId,
    pub title: NonEmptyString,
    pub body: String,
}

#[derive(Clone, Copy, From, AsRef, Deref, Debug, PartialEq)]
pub struct CommentId(usize);

pub struct Comment {
    pub id: CommentId,
    pub user_id: UserId,
    pub body: NonEmptyString,
}

#[derive(Clone, Copy, From, AsRef, Deref, PartialEq, Serialize, Debug)]
pub struct LabelId(usize);

#[derive(Serialize, Debug, PartialEq)]
pub struct Label {
    pub id: LabelId,
    pub name: NonEmptyString,
    pub description: String,
}
