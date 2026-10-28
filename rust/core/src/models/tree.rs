use crate::models::{hast, mdast};

/// The tree a pipeline stage takes or returns.
#[derive(Debug, PartialEq)]
pub enum Tree<'a> {
    Mdast(mdast::Node<'a>),
    Hast(hast::Node<'a>),
}

/// Which kind of tree a stage takes or returns, checked when a processor freezes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeKind {
    Mdast,
    Hast,
}

impl Tree<'_> {
    pub fn kind(&self) -> TreeKind {
        match self {
            Tree::Mdast(_) => TreeKind::Mdast,
            Tree::Hast(_) => TreeKind::Hast,
        }
    }
}
