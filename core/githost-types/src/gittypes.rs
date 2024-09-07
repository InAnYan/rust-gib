use non_empty_string::NonEmptyString;
use nutype::nutype;

#[nutype(derive(Clone, Copy, From, AsRef, Deref))]
pub struct UserId(usize);

pub struct User {
    pub id: UserId,
    pub nickname: NonEmptyString,
}

#[nutype(derive(Clone, Copy, From, AsRef, Deref))]
pub struct RepoId(usize);

pub struct Repo {
    pub id: RepoId,
    pub owner: String,
    pub name: String,
}

#[nutype(derive(Clone, Copy, From, AsRef, Deref))]
pub struct IssueId(usize);

pub struct Issue {
    pub id: IssueId,
    pub body: NonEmptyString,
}

#[nutype(derive(Clone, Copy, From, AsRef, Deref))]
pub struct CommentId(usize);

pub struct Comment {
    pub id: CommentId,
    pub user_id: UserId,
    pub body: NonEmptyString,
}

#[nutype(derive(Clone, Copy, From, AsRef, Deref, PartialEq))]
pub struct LabelId(usize);

pub struct Label {
    pub id: LabelId,
    pub name: NonEmptyString,
    pub description: String,
}
