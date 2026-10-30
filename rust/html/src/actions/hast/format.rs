use textlane_core::hast::{Node, Text};
use textlane_core::{Allocator, ArenaVec};

use crate::actions::hast::minify::minify_hast;
use crate::helpers::phrasing::{is_embedded, is_phrasing};
use crate::models::options::{Format, Indent};

// Indentation grows with depth on every line, so a deep tree would format into output
// quadratic in its size; deeper levels indent as this one, unlike hast-util-format
const MAX_INDENT_LEVEL: usize = 100;

/// Format whitespace in a hast tree for reading, as hast-util-format does: collapse it,
/// then put each block on its own line, indented by how deep it nests.
pub fn format_hast<'a>(
    tree: Node<'a>,
    options: &Format,
    alloc: &'a Allocator,
) -> Node<'a> {
    let indent = match &options.indent {
        Some(Indent::Spaces(size)) => " ".repeat(*size),
        Some(Indent::Text(text)) => text.clone(),
        None => "  ".to_string(),
    };
    let mut formatter = Formatter {
        blanks: &options.blanks,
        head: false,
        indent_initial: options.indent_initial != Some(false),
        indent,
        alloc,
    };
    let mut tree = tree;
    minify_hast(&mut tree, true, alloc);
    // Each frame holds the children it walks, taken out of their parent and put back once
    // walked, so deep trees need no recursion
    let mut frames = Vec::new();
    if formatter.visit(&mut tree, 0)
        && let Some(children) = children_of(&mut tree)
    {
        frames.push(Frame {
            children: std::mem::replace(children, ArenaVec::new_in(alloc)),
            index: 0,
            depth: 1,
        });
    }
    while let Some(frame) = frames.last_mut() {
        let (index, depth) = (frame.index, frame.depth);
        frame.index += 1;
        let Some(child) = frame.children.get_mut(index) else {
            let Some(done) = frames.pop() else {
                break;
            };
            let parent = match frames.last_mut() {
                Some(parent) => parent.children.get_mut(parent.index - 1),
                None => Some(&mut tree),
            };
            if let Some(children) = parent.and_then(children_of) {
                *children = done.children;
            }
            continue;
        };
        if formatter.visit(child, depth)
            && let Some(children) = children_of(child)
        {
            let children = std::mem::replace(children, ArenaVec::new_in(alloc));
            frames.push(Frame {
                children,
                index: 0,
                depth: depth + 1,
            });
        }
    }
    tree
}

struct Frame<'a> {
    children: ArenaVec<'a, Node<'a>>,
    index: usize,
    depth: usize,
}

struct Formatter<'o, 'a> {
    blanks: &'o [String],
    head: bool,
    indent_initial: bool,
    indent: String,
    alloc: &'a Allocator,
}

impl<'a> Formatter<'_, 'a> {
    // hast-util-format's visitor; whether to walk the node's children
    fn visit(&mut self, node: &mut Node<'a>, depth: usize) -> bool {
        if let Node::Element(element) = node {
            match element.tag_name {
                "head" => self.head = true,
                "body" => self.head = false,
                "pre" | "script" | "style" | "textarea" => return false,
                _ => {}
            }
        }
        let padded = self.padding(node);
        let Some(children) = children_of(node) else {
            return false;
        };
        if children.is_empty() || !padded {
            return true;
        }
        let level = if self.indent_initial {
            depth as isize
        } else {
            depth as isize - 1
        };
        let indent = self
            .indent
            .repeat(level.max(0).unsigned_abs().min(MAX_INDENT_LEVEL));
        let mut eol = false;
        for child in children.iter_mut() {
            let value = match child {
                Node::Text(text) => &mut text.value,
                Node::Comment(comment) => &mut comment.value,
                _ => continue,
            };
            if value.contains('\n') {
                eol = true;
                *value = self
                    .alloc
                    .alloc_str(&value.replace('\n', &format!("\n{indent}")));
            }
        }
        let mut old = std::mem::replace(children, ArenaVec::new_in(self.alloc));
        let mut result = ArenaVec::with_capacity_in(old.len() * 2 + 1, self.alloc);
        for child in old.drain(..) {
            if self.padding(&child) || (eol && result.is_empty()) {
                self.add_break(&mut result, level, Some(&child));
                eol = true;
            }
            result.push(child);
        }
        if result
            .last()
            .is_some_and(|previous| eol || self.padding(previous))
        {
            if result.last().is_some_and(is_whitespace) {
                result.pop();
            }
            self.add_break(&mut result, level - 1, None);
        }
        *children = result;
        true
    }

    fn add_break(
        &self,
        list: &mut ArenaVec<'a, Node<'a>>,
        level: isize,
        next: Option<&Node<'a>>,
    ) {
        let length = list.len();
        let previous = match list.last() {
            Some(tail) if is_whitespace(tail) => {
                length.checked_sub(2).map(|at| &list[at])
            }
            tail => tail,
        };
        let mut replace =
            String::from(if self.is_blank(previous) && self.is_blank(next) {
                "\n\n"
            } else {
                "\n"
            });
        replace.push_str(
            &self
                .indent
                .repeat(level.max(0).unsigned_abs().min(MAX_INDENT_LEVEL)),
        );
        match list.last_mut() {
            Some(Node::Text(tail)) => {
                tail.value = if is_whitespace_text(tail.value) {
                    self.alloc.alloc_str(&replace)
                } else {
                    self.alloc.alloc_str(&format!("{}{replace}", tail.value))
                };
            }
            _ => list.push(Node::Text(Text {
                value: self.alloc.alloc_str(&replace),
                position: None,
                data: None,
            })),
        }
    }

    fn is_blank(&self, node: Option<&Node<'_>>) -> bool {
        match node {
            Some(Node::Element(element)) => {
                self.blanks.iter().any(|blank| blank == element.tag_name)
            }
            _ => false,
        }
    }

    fn padding(&self, node: &Node<'_>) -> bool {
        match node {
            Node::Root(_) => true,
            Node::Element(element) => {
                self.head
                    || element.tag_name == "script"
                    || is_embedded(element)
                    || !is_phrasing(node)
            }
            _ => false,
        }
    }
}

fn children_of<'n, 'a>(node: &'n mut Node<'a>) -> Option<&'n mut ArenaVec<'a, Node<'a>>> {
    match node {
        Node::Root(root) => Some(&mut root.children),
        Node::Element(element) => Some(&mut element.children),
        Node::Unknown(unknown) => unknown.children.as_mut(),
        Node::MdxJsxFlowElement(element) | Node::MdxJsxTextElement(element) => {
            Some(&mut element.children)
        }
        _ => None,
    }
}

fn is_whitespace(node: &Node<'_>) -> bool {
    matches!(node, Node::Text(text) if is_whitespace_text(text.value))
}

// hast-util-whitespace: only `[ \t\n\f\r]`, or nothing
fn is_whitespace_text(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| matches!(byte, b' ' | b'\t' | b'\n' | 0x0C | b'\r'))
}
