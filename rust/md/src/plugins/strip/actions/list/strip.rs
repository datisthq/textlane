use textlane_core::mdast::{Node, Paragraph, Text};
use textlane_core::{Allocator, ArenaString, ArenaVec, Error};

use crate::models::options::Strip;
use crate::utils::serializer::state::read_node_type;

/// What strip-markdown does with a node of a type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Handler {
    Children,
    Empty,
    Paragraph,
    Text,
    Image,
    Break,
}

const DEFAULTS: &[(&str, Handler)] = &[
    ("blockquote", Handler::Children),
    ("break", Handler::Break),
    ("code", Handler::Empty),
    ("definition", Handler::Empty),
    ("delete", Handler::Children),
    ("emphasis", Handler::Children),
    ("footnoteReference", Handler::Empty),
    ("footnoteDefinition", Handler::Empty),
    ("heading", Handler::Paragraph),
    ("html", Handler::Empty),
    ("image", Handler::Image),
    ("imageReference", Handler::Image),
    ("inlineCode", Handler::Text),
    ("list", Handler::Children),
    ("listItem", Handler::Children),
    ("link", Handler::Children),
    ("linkReference", Handler::Children),
    ("strong", Handler::Children),
    ("table", Handler::Empty),
    ("tableCell", Handler::Empty),
    ("text", Handler::Text),
    ("thematicBreak", Handler::Empty),
    ("toml", Handler::Empty),
    ("yaml", Handler::Empty),
];

/// The handler of each node type after `remove` and `keep`, failing as strip-markdown does
/// on a kept type it has no handler for.
pub(crate) fn read_handlers(options: &Strip) -> Result<Vec<(String, Handler)>, Error> {
    let mut handlers: Vec<(String, Handler)> = DEFAULTS
        .iter()
        .map(|(kind, handler)| ((*kind).to_owned(), *handler))
        .collect();
    for kind in &options.remove {
        match handlers.iter_mut().find(|(known, _)| known == kind) {
            Some(entry) => entry.1 = Handler::Empty,
            None => handlers.push((kind.clone(), Handler::Empty)),
        }
    }
    for kind in &options.keep {
        if !handlers.iter().any(|(known, _)| known == kind) {
            let message = format!(
                "Unknown node type `{kind}` in `keep`, use a replace tuple with a handle instead: `remove: [['{kind}', handle]]`"
            );
            return Err(Error::new("invalid-options", message));
        }
    }
    handlers.retain(|(kind, _)| !options.keep.contains(kind));
    Ok(handlers)
}

/// Strip one list of siblings: each node by its handler, the children a handler hands back
/// stripped in its place, then adjacent texts joined.
pub(crate) fn strip_list<'a>(
    children: &mut ArenaVec<'a, Node<'a>>,
    handlers: &[(String, Handler)],
    alloc: &'a Allocator,
) {
    let nodes = std::mem::replace(children, ArenaVec::new_in(alloc));
    let mut pending: Vec<Node<'a>> = nodes.into_iter().rev().collect();
    let mut stripped: Vec<Node<'a>> = Vec::with_capacity(pending.len());
    while let Some(mut node) = pending.pop() {
        let kind = read_node_type(&node);
        let handler = handlers
            .iter()
            .find(|(known, _)| known == kind)
            .map(|(_, h)| *h);
        match handler {
            None => stripped.push(node),
            Some(Handler::Empty) => {}
            Some(Handler::Children) => {
                if let Some(children) = node.children_mut() {
                    let children = std::mem::replace(children, ArenaVec::new_in(alloc));
                    pending.extend(children.into_iter().rev());
                }
            }
            Some(Handler::Paragraph) => {
                let children = match node.children_mut() {
                    Some(children) => {
                        std::mem::replace(children, ArenaVec::new_in(alloc))
                    }
                    None => ArenaVec::new_in(alloc),
                };
                stripped.push(Node::Paragraph(Paragraph {
                    children,
                    position: None,
                    data: None,
                }));
            }
            Some(handler) => {
                if let Some(value) = read_value(&node, handler) {
                    stripped.push(create_text(value));
                }
            }
        }
    }
    join_texts(stripped, children, alloc);
}

// `text` and `inlineCode` give their value, an image its alt or title, a break a line ending
fn read_value<'a>(node: &Node<'a>, handler: Handler) -> Option<&'a str> {
    match (handler, node) {
        (Handler::Break, _) => Some("\n"),
        (Handler::Image, Node::Image(image)) => image
            .alt
            .filter(|alt| !alt.is_empty())
            .or(image.title)
            .filter(|v| !v.is_empty()),
        (Handler::Image, Node::ImageReference(image)) => {
            image.alt.filter(|alt| !alt.is_empty())
        }
        (Handler::Text, Node::Text(text)) => Some(text.value),
        (Handler::Text, Node::InlineCode(code)) => Some(code.value),
        (Handler::Text, Node::InlineMath(math)) => Some(math.value),
        (Handler::Text, Node::Html(html)) => Some(html.value),
        (Handler::Text, Node::Code(code)) => Some(code.value),
        (Handler::Text, Node::Math(math)) => Some(math.value),
        _ => None,
    }
}

fn create_text(value: &str) -> Node<'_> {
    Node::Text(Text {
        value,
        position: None,
        data: None,
    })
}

// strip-markdown's `clean`: a node of a valued type joins the one before when that is
// of the same type
fn join_texts<'a>(
    nodes: Vec<Node<'a>>,
    children: &mut ArenaVec<'a, Node<'a>>,
    alloc: &'a Allocator,
) {
    for node in nodes {
        let joined = match (children.last_mut(), &node) {
            (Some(Node::Text(previous)), Node::Text(next)) => {
                previous.value = join(previous.value, next.value, alloc);
                true
            }
            (Some(Node::InlineCode(previous)), Node::InlineCode(next)) => {
                previous.value = join(previous.value, next.value, alloc);
                true
            }
            (Some(Node::InlineMath(previous)), Node::InlineMath(next)) => {
                previous.value = join(previous.value, next.value, alloc);
                true
            }
            _ => false,
        };
        if !joined {
            children.push(node);
        }
    }
}

fn join<'a>(first: &str, second: &str, alloc: &'a Allocator) -> &'a str {
    let mut joined = ArenaString::with_capacity_in(first.len() + second.len(), alloc);
    joined.push_str(first);
    joined.push_str(second);
    joined.into_bump_str()
}
