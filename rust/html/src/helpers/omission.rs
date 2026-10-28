use textlane_core::hast::{Element, Node};

/// The parent a node is serialized in: its tag name when an element, and its children.
#[derive(Clone, Copy)]
pub(crate) struct Siblings<'n, 'a> {
    pub(crate) tag_name: Option<&'a str>,
    pub(crate) children: &'n [Node<'a>],
}

/// Whether hast-util-to-html leaves out `element`'s opening tag.
pub(crate) fn is_opening_omitted(
    element: &Element,
    index: usize,
    parent: Option<Siblings>,
) -> bool {
    let first =
        |include_whitespace| find_after(&element.children, None, include_whitespace);
    match element.tag_name {
        "html" => !first(false).is_some_and(is_comment),
        "head" => {
            let mut base = false;
            let mut title = false;
            for child in element.children.iter() {
                match child {
                    Node::Element(child) if child.tag_name == "base" => {
                        if base {
                            return false;
                        }
                        base = true;
                    }
                    Node::Element(child) if child.tag_name == "title" => {
                        if title {
                            return false;
                        }
                        title = true;
                    }
                    _ => {}
                }
            }
            element
                .children
                .first()
                .is_none_or(|child| matches!(child, Node::Element(_)))
        }
        "body" => first(true).is_none_or(|head| match head {
            Node::Comment(_) => false,
            Node::Text(text) => !is_whitespace_start(text.value),
            Node::Element(head) => !matches!(
                head.tag_name,
                "meta" | "link" | "script" | "style" | "template"
            ),
            _ => true,
        }),
        "colgroup" | "tbody" => {
            if let Some(parent) = parent
                && let Some(previous) = find_before(parent.children, index)
                && let Node::Element(previous_element) = &parent.children[previous]
            {
                let closes_before = if element.tag_name == "colgroup" {
                    previous_element.tag_name == "colgroup"
                } else {
                    matches!(previous_element.tag_name, "thead" | "tbody")
                };
                if closes_before
                    && is_closing_omitted(previous_element, previous, Some(parent))
                {
                    return false;
                }
            }
            let wanted = if element.tag_name == "colgroup" {
                "col"
            } else {
                "tr"
            };
            first(element.tag_name == "colgroup")
                .is_some_and(|head| is_element(head, &[wanted]))
        }
        _ => false,
    }
}

/// Whether hast-util-to-html leaves out `element`'s closing tag.
pub(crate) fn is_closing_omitted(
    element: &Element,
    index: usize,
    parent: Option<Siblings>,
) -> bool {
    let children = parent.map_or(&[][..], |parent| parent.children);
    let next = |include_whitespace| find_after(children, Some(index), include_whitespace);
    match element.tag_name {
        "caption" | "colgroup" | "head" => next(true).is_none_or(|next| match next {
            Node::Comment(_) => false,
            Node::Text(text) => !is_whitespace_start(text.value),
            _ => true,
        }),
        "html" | "body" => !next(false).is_some_and(is_comment),
        "p" => match next(false) {
            Some(next) => is_element(
                next,
                &[
                    "address",
                    "article",
                    "aside",
                    "blockquote",
                    "details",
                    "div",
                    "dl",
                    "fieldset",
                    "figcaption",
                    "figure",
                    "footer",
                    "form",
                    "h1",
                    "h2",
                    "h3",
                    "h4",
                    "h5",
                    "h6",
                    "header",
                    "hgroup",
                    "hr",
                    "main",
                    "menu",
                    "nav",
                    "ol",
                    "p",
                    "pre",
                    "section",
                    "table",
                    "ul",
                ],
            ),
            None => !parent
                .and_then(|parent| parent.tag_name)
                .is_some_and(|name| {
                    matches!(
                        name,
                        "a" | "audio" | "del" | "ins" | "map" | "noscript" | "video"
                    )
                }),
        },
        "li" => next(false).is_none_or(|next| is_element(next, &["li"])),
        "dt" => next(false).is_some_and(|next| is_element(next, &["dt", "dd"])),
        "dd" => next(false).is_none_or(|next| is_element(next, &["dt", "dd"])),
        "rp" | "rt" => next(false).is_none_or(|next| is_element(next, &["rp", "rt"])),
        "optgroup" => next(false).is_none_or(|next| is_element(next, &["optgroup"])),
        "option" => {
            next(false).is_none_or(|next| is_element(next, &["option", "optgroup"]))
        }
        "thead" => next(false).is_some_and(|next| is_element(next, &["tbody", "tfoot"])),
        "tbody" => next(false).is_none_or(|next| is_element(next, &["tbody", "tfoot"])),
        "tfoot" => next(false).is_none(),
        "tr" => next(false).is_none_or(|next| is_element(next, &["tr"])),
        "td" | "th" => next(false).is_none_or(|next| is_element(next, &["td", "th"])),
        _ => false,
    }
}

fn find_after<'n, 'a>(
    children: &'n [Node<'a>],
    index: Option<usize>,
    include_whitespace: bool,
) -> Option<&'n Node<'a>> {
    let start = index.map_or(0, |index| index + 1);
    children
        .get(start..)?
        .iter()
        .find(|node| include_whitespace || !is_whitespace(node))
}

fn find_before(children: &[Node], index: usize) -> Option<usize> {
    (0..index)
        .rev()
        .find(|index| !is_whitespace(&children[*index]))
}

fn is_comment(node: &Node) -> bool {
    matches!(node, Node::Comment(_))
}

fn is_element(node: &Node, names: &[&str]) -> bool {
    matches!(node, Node::Element(element) if names.contains(&element.tag_name))
}

// hast-util-whitespace: text of only HTML whitespace, or none
fn is_whitespace(node: &Node) -> bool {
    matches!(node, Node::Text(text) if is_inter_element_whitespace(text.value))
}

fn is_inter_element_whitespace(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b'\x0C'))
}

// An empty first character counts as whitespace, as hast-util-whitespace reads `''`
fn is_whitespace_start(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_none_or(|first| matches!(first, ' ' | '\t' | '\n' | '\r' | '\x0C'))
}
