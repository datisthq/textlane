use textlane_core::ArenaVec;
use textlane_core::nlcst::Node;

pub(crate) fn children_mut<'n, 'a>(
    node: &'n mut Node<'a>,
) -> Option<&'n mut ArenaVec<'a, Node<'a>>> {
    match node {
        Node::Root(parent)
        | Node::Paragraph(parent)
        | Node::Sentence(parent)
        | Node::Word(parent) => Some(&mut parent.children),
        Node::Unknown(unknown) => unknown.children.as_mut(),
        _ => None,
    }
}

/// Punctuation or a symbol: the nodes typography rewrites.
pub(crate) fn is_mark(node: &Node<'_>) -> bool {
    matches!(node, Node::Punctuation(_) | Node::Symbol(_))
}

pub(crate) fn mark_value<'a>(node: &Node<'a>) -> Option<&'a str> {
    match node {
        Node::Punctuation(literal) | Node::Symbol(literal) => Some(literal.value),
        _ => None,
    }
}

pub(crate) fn set_value<'a>(node: &mut Node<'a>, value: &'a str) {
    if let Node::Punctuation(literal) | Node::Symbol(literal) = node {
        literal.value = value;
    }
}

/// Whether text is a decade abbreviation, as `/^\d\ds$/` tests it: `80s`.
pub(crate) fn is_decade(value: &str) -> bool {
    matches!(value.as_bytes(), [first, second, b's'] if first.is_ascii_digit() && second.is_ascii_digit())
}
