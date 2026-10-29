use std::borrow::Cow;
use std::collections::HashMap;
use std::mem::discriminant;

use textlane_core::hast::{Element, MdxJsxElement, Node, Root, Unknown};
use textlane_core::{Allocator, ArenaVec, Error};

use crate::models::options::RawOptions;
use crate::utils::builder::convert::convert_dom;
use crate::utils::builder::dom::{Dom, NodeId};
use crate::utils::builder::parser::Parser;

// Node types of MDX, which hast-util-raw names in its error
const MDX_TYPES: [&str; 5] = [
    "mdxFlowExpression",
    "mdxJsxFlowElement",
    "mdxJsxTextElement",
    "mdxTextExpression",
    "mdxjsEsm",
];

/// Pass a hast tree through the HTML parser, as hast-util-raw does: raw nodes become real
/// nodes and nesting is fixed.
pub fn raw_hast<'a>(
    tree: &Node<'a>,
    options: &RawOptions,
    alloc: &'a Allocator,
) -> Result<Node<'a>, Error> {
    let head = match tree {
        Node::Root(root) => root.children.first(),
        node => Some(node),
    };
    let mut reparse = Reparse {
        options,
        alloc,
        jobs: vec![(std::slice::from_ref(tree), is_document(head))],
        stitches: Vec::new(),
    };
    // A passed-through parent's children are parsed on their own, as upstream recurses;
    // as jobs in a queue, deep nesting never recurses
    let mut parsed = Vec::new();
    let mut index = 0;
    while let Some((nodes, document)) = reparse.jobs.get(index).copied() {
        parsed.push(reparse.parse(nodes, document)?);
        index += 1;
    }
    let mut results: Vec<Option<ArenaVec<'a, Node<'a>>>> = Vec::new();
    results.resize_with(parsed.len(), || None);
    let mut top = Node::Root(Root {
        children: ArenaVec::new_in(alloc),
        position: None,
        data: None,
        quirks_mode: Some(false),
    });
    // A job's stitches only name later jobs, so converting backwards finds them done
    for (index, job) in parsed.into_iter().enumerate().rev() {
        let mut replacements = HashMap::new();
        for (comment, stitch) in job.stitches {
            let (node, child) = reparse.stitches[stitch];
            let children = read_children(node).map(|_| {
                child
                    .and_then(|child| results[child].take())
                    .unwrap_or_else(|| ArenaVec::new_in(alloc))
            });
            replacements.insert(comment, rebuild(node, children));
        }
        let node = convert_dom(&job.dom, job.root, 0, alloc, &mut replacements, None);
        if index == 0 {
            top = node;
        } else if let Node::Root(root) = node {
            results[index] = Some(root.children);
        }
    }
    let Node::Root(mut root) = top else {
        return Ok(top);
    };
    // Unpacked when not given a root, as upstream
    if root.children.len() == 1
        && root
            .children
            .first()
            .is_some_and(|child| is_same_type(child, tree))
        && let Some(child) = root.children.pop()
    {
        return Ok(child);
    }
    Ok(Node::Root(root))
}

struct Reparse<'n, 'o, 'a> {
    options: &'o RawOptions,
    alloc: &'a Allocator,
    jobs: Vec<(&'n [Node<'a>], bool)>,
    // Each passed-through node, and the job parsing its children
    stitches: Vec<(&'n Node<'a>, Option<usize>)>,
}

struct Parsed {
    dom: Dom,
    root: NodeId,
    // The comment each stitch was sent through the parser as
    stitches: Vec<(NodeId, usize)>,
}

impl<'n, 'a> Reparse<'n, '_, 'a> {
    fn parse(&mut self, nodes: &'n [Node<'a>], document: bool) -> Result<Parsed, Error> {
        let mut parser = if document {
            Parser::for_document("")
        } else {
            Parser::for_fragment("")
        };
        let mut stitches = Vec::new();
        let mut stack: Vec<(std::slice::Iter<'n, Node<'a>>, Option<&'n Element<'a>>)> =
            vec![(nodes.iter(), None)];
        while let Some((children, _)) = stack.last_mut() {
            let Some(node) = children.next() else {
                if let Some((_, Some(element))) = stack.pop() {
                    parser.feed_end_tag(element);
                }
                continue;
            };
            match node {
                Node::Root(root) => stack.push((root.children.iter(), None)),
                Node::Element(element) => {
                    parser.feed_start_tag(element);
                    stack.push((element.children.iter(), Some(element)));
                }
                Node::Text(text) => parser.feed_text(text.value),
                Node::Comment(comment) => {
                    parser.feed_comment(comment.value);
                }
                Node::Doctype(_) => parser.feed_doctype(),
                Node::Raw(raw) => {
                    let value = match self.options.tagfilter {
                        true => filter_tags(raw.value, self.alloc),
                        false => raw.value,
                    };
                    parser.feed_raw(value, raw.position.is_some());
                }
                node if read_kind(node).is_some_and(|kind| {
                    self.options.pass_through.iter().any(|pass| pass == kind)
                }) =>
                {
                    let job = read_children(node).map(|children| {
                        let head = children.first();
                        self.jobs.push((children, is_document(head)));
                        self.jobs.len() - 1
                    });
                    self.stitches.push((node, job));
                    if let Some(comment) = parser.feed_comment("") {
                        stitches.push((comment, self.stitches.len() - 1));
                    }
                }
                node => {
                    let kind = read_kind(node).unwrap_or_default();
                    let extra = if MDX_TYPES.contains(&kind) {
                        ". It looks like you are using MDX nodes with `hast-util-raw` (or `rehype-raw`). If you use this because you are using remark or rehype plugins that inject `'html'` nodes, then please raise an issue with that plugin, as its a bad and slow idea. If you use this because you are using markdown syntax, then you have to configure this utility (or plugin) to pass through these nodes (see `passThrough` in docs), but you can also migrate to use the MDX syntax"
                    } else {
                        ""
                    };
                    return Err(Error::new(
                        "unknown-node",
                        format!("Cannot compile `{kind}` node{extra}"),
                    ));
                }
            }
        }
        parser.finish_feed();
        let root = if document {
            parser.document
        } else {
            parser.take_fragment()
        };
        Ok(Parsed {
            dom: std::mem::take(&mut parser.dom),
            root,
            stitches,
        })
    }
}

// Whether a tree is a whole document: it starts with a doctype or `<html>`
// The type of a node hast does not define, which only `passThrough` lets through
fn read_kind<'a>(node: &Node<'a>) -> Option<&'a str> {
    match node {
        Node::Unknown(unknown) => Some(unknown.kind),
        Node::MdxJsxFlowElement(_) => Some("mdxJsxFlowElement"),
        Node::MdxJsxTextElement(_) => Some("mdxJsxTextElement"),
        Node::MdxFlowExpression(_) => Some("mdxFlowExpression"),
        Node::MdxTextExpression(_) => Some("mdxTextExpression"),
        Node::MdxjsEsm(_) => Some("mdxjsEsm"),
        _ => None,
    }
}

fn read_children<'n, 'a>(node: &'n Node<'a>) -> Option<&'n [Node<'a>]> {
    match node {
        Node::Unknown(unknown) => unknown.children.as_ref().map(|children| &children[..]),
        Node::MdxJsxFlowElement(element) | Node::MdxJsxTextElement(element) => {
            Some(&element.children)
        }
        _ => None,
    }
}

// A passed-through node again, with its children reparsed
fn rebuild<'a>(node: &Node<'a>, children: Option<ArenaVec<'a, Node<'a>>>) -> Node<'a> {
    let element = |element: &MdxJsxElement<'a, Node<'a>>,
                   children: Option<ArenaVec<'a, Node<'a>>>| {
        let alloc = element.attributes.bump();
        let mut attributes = ArenaVec::with_capacity_in(element.attributes.len(), alloc);
        attributes.extend(element.attributes.iter().cloned());
        MdxJsxElement {
            name: element.name,
            attributes,
            children: children.unwrap_or_else(|| ArenaVec::new_in(alloc)),
            position: element.position,
            data: element.data,
        }
    };
    match node {
        Node::MdxJsxFlowElement(node) => Node::MdxJsxFlowElement(element(node, children)),
        Node::MdxJsxTextElement(node) => Node::MdxJsxTextElement(element(node, children)),
        Node::MdxFlowExpression(node) => Node::MdxFlowExpression(node.clone()),
        Node::MdxTextExpression(node) => Node::MdxTextExpression(node.clone()),
        Node::MdxjsEsm(node) => Node::MdxjsEsm(node.clone()),
        Node::Unknown(node) => Node::Unknown(Unknown { children, ..*node }),
        _ => Node::Unknown(Unknown {
            kind: "",
            fields: &[],
            children,
            position: None,
            data: None,
        }),
    }
}

fn is_document(head: Option<&Node<'_>>) -> bool {
    match head {
        Some(Node::Doctype(_)) => true,
        Some(Node::Element(element)) => element.tag_name.eq_ignore_ascii_case("html"),
        _ => false,
    }
}

fn is_same_type(left: &Node<'_>, right: &Node<'_>) -> bool {
    match (left, right) {
        (Node::Unknown(_), _) | (_, Node::Unknown(_)) => {
            read_kind(left) == read_kind(right)
        }
        _ => discriminant(left) == discriminant(right),
    }
}

// GFM's tagfilter, `/<(\/?)(iframe|…|xmp)(?=[\t\n\f\r />])/gi`, opening `<` escaped
fn filter_tags<'a>(value: &'a str, alloc: &'a Allocator) -> &'a str {
    const NAMES: [&str; 9] = [
        "iframe",
        "noembed",
        "noframes",
        "plaintext",
        "script",
        "style",
        "textarea",
        "title",
        "xmp",
    ];
    let bytes = value.as_bytes();
    let mut output = Cow::Borrowed("");
    let mut copied = 0;
    for (index, _) in value.match_indices('<') {
        let mut name_start = index + 1;
        if bytes.get(name_start) == Some(&b'/') {
            name_start += 1;
        }
        let matched = NAMES.iter().any(|name| {
            let end = name_start + name.len();
            bytes
                .get(name_start..end)
                .is_some_and(|slice| slice.eq_ignore_ascii_case(name.as_bytes()))
                && bytes.get(end).is_some_and(|next| {
                    matches!(next, b'\t' | b'\n' | 0x0C | b'\r' | b' ' | b'/' | b'>')
                })
        });
        if matched {
            let output = output.to_mut();
            output.push_str(&value[copied..index]);
            output.push_str("&lt;");
            copied = index + 1;
        }
    }
    if copied == 0 {
        return value;
    }
    let output = output.to_mut();
    output.push_str(&value[copied..]);
    alloc.alloc_str(output)
}

#[cfg(test)]
#[path = "raw_unit.rs"]
mod unit;
