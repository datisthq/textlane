use std::borrow::Cow;

use textlane_core::hast::{Element, Node, Text};
use textlane_core::{Allocator, ArenaVec};

use crate::helpers::phrasing::is_embedded;
use crate::helpers::value::{find_value, is_truthy};

// The walk recurses once per level; past the red zone the stack grows on the heap
const STACK_RED_ZONE: usize = 64 * 1024;
const STACK_GROWTH: usize = 1024 * 1024;

/// Collapse whitespace as rehype-minify-whitespace does; with `newlines`, a run holding a
/// line ending collapses to it instead of a space.
pub(crate) fn minify_hast<'a>(node: &mut Node<'a>, newlines: bool, alloc: &'a Allocator) {
    let mut minifier = Minifier {
        newlines,
        alloc,
        boundaries: vec![],
        sizes: vec![],
    };
    minifier.measure(node);
    let state = State {
        whitespace: Whitespace::Normal,
        before: false,
        after: false,
    };
    minifier.minify(node, 0, state);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Whitespace {
    Normal,
    Pre,
    PreWrap,
    Nowrap,
}

#[derive(Clone, Copy)]
struct State {
    whitespace: Whitespace,
    before: bool,
    after: bool,
}

#[derive(Default)]
struct Outcome {
    ignore: bool,
    strip_at_start: bool,
    remove: bool,
}

struct Minifier<'a> {
    newlines: bool,
    alloc: &'a Allocator,
    /// By the node's index in document order: whether whitespace before it collapses,
    /// as `collapsableAfter` finds by searching it, worked out once for every node so
    /// that search is not repeated for each sibling before it.
    boundaries: Vec<Option<bool>>,
    /// By the node's index in document order: how many nodes its subtree holds.
    sizes: Vec<usize>,
}

impl<'a> Minifier<'a> {
    fn measure(&mut self, node: &Node<'_>) {
        let index = self.boundaries.len();
        self.boundaries.push(None);
        self.sizes.push(1);
        let children = children_of(node);
        let mut first = None;
        for child in children.unwrap_or_default() {
            let child_index = self.boundaries.len();
            stacker::maybe_grow(STACK_RED_ZONE, STACK_GROWTH, || self.measure(child));
            first = first.or(self.boundaries[child_index]);
        }
        self.sizes[index] = self.boundaries.len() - index;
        self.boundaries[index] = infer_boundary(node).or_else(|| {
            (children.is_some() && !is_skippable(node))
                .then_some(first)
                .flatten()
        });
    }

    fn minify(&self, node: &mut Node<'a>, index: usize, state: State) -> Outcome {
        match node {
            Node::Root(root) => {
                let settings = State {
                    before: true,
                    after: true,
                    ..state
                };
                self.all(&mut root.children, index, settings)
            }
            Node::Element(element) => {
                let mut settings = state;
                if is_blocklike(element.tag_name) {
                    settings.before = true;
                    settings.after = true;
                }
                settings.whitespace = infer_whitespace(element, state.whitespace);
                self.all(&mut element.children, index, settings)
            }
            Node::Text(text) => match state.whitespace {
                Whitespace::Normal => self.minify_text(text, state),
                Whitespace::Nowrap => {
                    if let Cow::Owned(value) = collapse(text.value, self.newlines) {
                        text.value = self.alloc.alloc_str(&value);
                    }
                    Outcome::default()
                }
                Whitespace::Pre | Whitespace::PreWrap => Outcome::default(),
            },
            Node::Comment(_) | Node::Doctype(_) => Outcome {
                ignore: true,
                ..Outcome::default()
            },
            Node::Raw(_)
            | Node::Unknown(_)
            | Node::MdxJsxFlowElement(_)
            | Node::MdxJsxTextElement(_)
            | Node::MdxFlowExpression(_)
            | Node::MdxTextExpression(_)
            | Node::MdxjsEsm(_) => Outcome::default(),
        }
    }

    fn minify_text(&self, text: &mut Text<'a>, state: State) -> Outcome {
        let value = collapse(text.value, self.newlines);
        let bytes = value.as_bytes();
        let mut result = Outcome::default();
        let mut start = 0;
        let mut end = bytes.len();
        if state.before && bytes.first().is_some_and(is_removable) {
            start += 1;
        }
        if start != end && bytes.get(end - 1).is_some_and(is_removable) {
            if state.after {
                end -= 1;
            } else {
                result.strip_at_start = true;
            }
        }
        if start == end {
            result.remove = true;
        } else if start != 0 || end != bytes.len() || matches!(value, Cow::Owned(_)) {
            text.value = self.alloc.alloc_str(&value[start..end]);
        }
        result
    }

    fn all(
        &self,
        children: &mut ArenaVec<'a, Node<'a>>,
        index: usize,
        state: State,
    ) -> Outcome {
        // Each child's index in document order, and whether whitespace collapses after it
        let mut indices = Vec::with_capacity(children.len());
        let mut next = index + 1;
        for _ in 0..children.len() {
            indices.push(next);
            next += self.sizes[next];
        }
        let mut collapsable = vec![None; children.len()];
        let mut found = None;
        for (child, index) in indices.iter().enumerate().rev() {
            collapsable[child] = found;
            found = self.boundaries[*index].or(found);
        }
        let mut before = state.before;
        let after = state.after;
        let mut removed = vec![false; children.len()];
        let mut last_kept = None;
        for child in 0..children.len() {
            let child_state = State {
                after: collapsable[child].unwrap_or(after),
                before,
                ..state
            };
            let result = stacker::maybe_grow(STACK_RED_ZONE, STACK_GROWTH, || {
                self.minify(&mut children[child], indices[child], child_state)
            });
            // Once a child is removed, upstream checks the one before it for content
            let current = if result.remove {
                removed[child] = true;
                last_kept
            } else {
                if !result.ignore {
                    before = result.strip_at_start;
                }
                last_kept = Some(child);
                last_kept
            };
            if current.is_some_and(|current| is_content(&children[current])) {
                before = false;
            }
        }
        if removed.contains(&true) {
            let mut removed = removed.into_iter();
            children.retain(|_| !removed.next().unwrap_or(false));
        }
        Outcome {
            strip_at_start: before || after,
            ..Outcome::default()
        }
    }
}

fn children_of<'n, 'a>(node: &'n Node<'a>) -> Option<&'n [Node<'a>]> {
    match node {
        Node::Root(root) => Some(&root.children),
        Node::Element(element) => Some(&element.children),
        _ => None,
    }
}

// Whether whitespace before a node collapses (`true`), stays (`false`), or depends on
// what follows (`None`)
fn infer_boundary(node: &Node<'_>) -> Option<bool> {
    match node {
        Node::Element(_) if is_content(node) => Some(false),
        Node::Element(element) if is_blocklike(element.tag_name) => Some(true),
        Node::Element(_) => None,
        Node::Text(text) if !is_whitespace(text.value) => Some(false),
        Node::Text(_) | Node::Comment(_) | Node::Doctype(_) => None,
        Node::Root(_)
        | Node::Raw(_)
        | Node::Unknown(_)
        | Node::MdxJsxFlowElement(_)
        | Node::MdxJsxTextElement(_)
        | Node::MdxFlowExpression(_)
        | Node::MdxTextExpression(_)
        | Node::MdxjsEsm(_) => Some(false),
    }
}

fn is_content(node: &Node<'_>) -> bool {
    match node {
        Node::Element(element) => {
            is_embedded(element)
                || matches!(element.tag_name, "button" | "input" | "select" | "textarea")
        }
        _ => false,
    }
}

fn is_skippable(node: &Node<'_>) -> bool {
    match node {
        Node::Element(element) => {
            is_truthy(find_value(element, "hidden"))
                || matches!(
                    element.tag_name,
                    "area"
                        | "base"
                        | "basefont"
                        | "dialog"
                        | "datalist"
                        | "head"
                        | "link"
                        | "meta"
                        | "noembed"
                        | "noframes"
                        | "param"
                        | "rp"
                        | "script"
                        | "source"
                        | "style"
                        | "template"
                        | "track"
                        | "title"
                )
        }
        Node::Comment(_) | Node::Doctype(_) => true,
        _ => false,
    }
}

fn is_blocklike(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "body"
            | "br"
            | "caption"
            | "center"
            | "col"
            | "colgroup"
            | "dd"
            | "dialog"
            | "dir"
            | "div"
            | "dl"
            | "dt"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "head"
            | "header"
            | "hgroup"
            | "hr"
            | "html"
            | "legend"
            | "li"
            | "listing"
            | "main"
            | "menu"
            | "nav"
            | "ol"
            | "optgroup"
            | "option"
            | "p"
            | "plaintext"
            | "pre"
            | "section"
            | "summary"
            | "table"
            | "tbody"
            | "td"
            | "tfoot"
            | "th"
            | "thead"
            | "tr"
            | "ul"
            | "wbr"
            | "xmp"
    )
}

fn infer_whitespace(element: &Element<'_>, inherited: Whitespace) -> Whitespace {
    match element.tag_name {
        "listing" | "plaintext" | "script" | "style" | "xmp" => Whitespace::Pre,
        "nobr" => Whitespace::Nowrap,
        "pre" if is_truthy(find_value(element, "wrap")) => Whitespace::PreWrap,
        "pre" => Whitespace::Pre,
        "td" | "th" if is_truthy(find_value(element, "noWrap")) => Whitespace::Nowrap,
        "textarea" => Whitespace::PreWrap,
        _ => inherited,
    }
}

// `/[\t\n\v\f\r ]+/g` replaced by a space, or with `newlines` by the run's first line ending
fn collapse(value: &str, newlines: bool) -> Cow<'_, str> {
    let is_space = |byte: &u8| matches!(byte, b'\t' | b'\n' | 0x0B | 0x0C | b'\r' | b' ');
    let bytes = value.as_bytes();
    let mut output = String::new();
    let mut copied = 0;
    let mut index = 0;
    while index < bytes.len() {
        if !is_space(&bytes[index]) {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && is_space(&bytes[index]) {
            index += 1;
        }
        let run = &value[start..index];
        let replacement = if newlines {
            match run.find(['\n', '\r']) {
                Some(at) if run[at..].starts_with("\r\n") => "\r\n",
                Some(at) => &run[at..at + 1],
                None => " ",
            }
        } else {
            " "
        };
        if run != replacement {
            output.push_str(&value[copied..start]);
            output.push_str(replacement);
            copied = index;
        }
    }
    if copied == 0 && output.is_empty() {
        return Cow::Borrowed(value);
    }
    output.push_str(&value[copied..]);
    Cow::Owned(output)
}

fn is_removable(byte: &u8) -> bool {
    *byte == b' ' || *byte == b'\n'
}

fn is_whitespace(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| matches!(byte, b' ' | b'\t' | b'\n' | 0x0C | b'\r'))
}
