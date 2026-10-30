/// A resolved scalar, as the `yaml` package's core schema gives it to JavaScript.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Scalar {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
}

#[derive(Debug)]
pub(crate) enum Item {
    Scalar(Scalar),
    Sequence(Vec<usize>),
    Mapping(Vec<(usize, usize)>),
    /// The node the alias's anchor named when it was read.
    Alias(usize),
}

#[derive(Debug)]
pub(crate) struct Node {
    pub(crate) item: Item,
    pub(crate) anchored: bool,
    /// How deep the value nests once aliases are expanded: 0 for a scalar.
    pub(crate) height: usize,
}

/// One YAML document as a flat node store, in document order.
#[derive(Debug, Default)]
pub(crate) struct Document {
    pub(crate) nodes: Vec<Node>,
    pub(crate) root: Option<usize>,
}

/// How deep a value may nest: a JSON value deeper than this would not survive the bindings
/// (Python 3.9's `json` stops near 1000 levels).
pub(crate) const MAX_DEPTH: usize = 512;

/// The error every YAML failure shares.
pub(crate) fn yaml_error(message: impl Into<String>) -> textlane_core::Error {
    textlane_core::Error::new("invalid-frontmatter", message)
}
