use nutype::nutype;

pub struct Repo {
    pub owner: String,
    pub name: String,
}

pub struct Issue {
    pub id: IssueId,
    pub body: String,
}

#[nutype(derive(From, AsRef, Deref))]
pub struct IssueId(usize);
