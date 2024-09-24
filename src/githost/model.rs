use derive_more::derive::{AsRef, Deref, From};
use non_empty_string::NonEmptyString;
use serde::Serialize;

#[derive(Clone, Copy, From, AsRef, Deref, PartialEq)]
pub struct UserId(usize);

pub struct User {
    pub id: UserId,
    pub nickname: NonEmptyString,
}

#[derive(Clone, Copy, From, AsRef, Deref, Debug)]
pub struct RepoId(usize);

pub struct Repo {
    pub id: RepoId,
    pub owner: String,
    pub name: String,
}

#[derive(Serialize, Clone, Copy, From, AsRef, Deref, Debug)]
#[serde(transparent)]
pub struct IssueId(usize);

pub struct Issue {
    pub id: IssueId,
    pub author_user_id: UserId,
    pub title: NonEmptyString,
    pub body: String,
}

#[derive(Clone, Copy, From, AsRef, Deref, Debug)]
pub struct CommentId(usize);

pub struct Comment {
    pub id: CommentId,
    pub user_id: UserId,
    pub body: NonEmptyString,
}

#[derive(Clone, Copy, From, AsRef, Deref, PartialEq, Serialize)]
pub struct LabelId(usize);

#[derive(Serialize)]
pub struct Label {
    pub id: LabelId,
    pub name: NonEmptyString,
    pub description: String,
}
