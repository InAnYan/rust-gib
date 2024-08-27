use crate::gittypes::Issue;

pub struct GitEvent {
    pub kind: GitEventKind,
    // `GitEvent` is made as a struct to future-proof it (*if I'm using the term correctly, I mean I
    // left it as a struct, so that, what if in future I need to add a field that every event has?
    // Time for example.*)
}

pub enum GitEventKind {
    NewIssue(Issue),
}
