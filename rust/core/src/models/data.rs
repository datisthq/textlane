use crate::ArenaVec;

/// unist's `data`: whatever JSON a plugin attached to a node, kept as it is.
#[derive(Debug, PartialEq)]
pub enum Data<'a> {
    Null,
    Boolean(bool),
    Number(f64),
    String(&'a str),
    Array(ArenaVec<'a, Data<'a>>),
    Object(ArenaVec<'a, (&'a str, Data<'a>)>),
}
