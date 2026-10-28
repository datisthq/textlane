use textlane_core::hast::{self, Element};
use textlane_core::mdast;
use textlane_core::{Allocator, ArenaVec, Position};

use crate::actions::blockquote::convert::convert_blockquote;
use crate::actions::code::convert::convert_code;
use crate::actions::emphasis::convert::convert_emphasis;
use crate::actions::hard_break::convert::convert_hard_break;
use crate::actions::heading::convert::convert_heading;
use crate::actions::inline_code::convert::convert_inline_code;
use crate::actions::list::convert::convert_list;
use crate::actions::list_item::convert::{convert_list_item, is_list_loose};
use crate::actions::paragraph::convert::convert_paragraph;
use crate::actions::root::convert::convert_root;
use crate::actions::strong::convert::convert_strong;
use crate::actions::text::convert::convert_text;
use crate::actions::thematic_break::convert::convert_thematic_break;
use crate::models::options::ToHtmlOptions;

/// Turns mdast into hast, as mdast-util-to-hast does.
pub(crate) struct Converter<'a, 'o> {
    pub(crate) alloc: &'a Allocator,
    // `allowDangerousHtml` decides what raw HTML becomes, once html nodes exist
    #[allow(dead_code)]
    pub(crate) options: &'o ToHtmlOptions,
    /// Whether each list being converted is loose, worked out once per list for its items.
    pub(crate) loose_lists: Vec<bool>,
}

// A parent whose children are being converted
struct Frame<'n, 'a> {
    node: &'n mdast::Node<'a>,
    next: usize,
    start: usize,
    trim: bool,
}

impl<'a> Converter<'a, '_> {
    /// Convert a node into none, one or several hast nodes.
    ///
    /// Walks with an explicit stack instead of recursing, so a deeply nested tree cannot
    /// overflow the call stack.
    pub(crate) fn convert(
        &mut self,
        node: &mdast::Node<'a>,
    ) -> ArenaVec<'a, hast::Node<'a>> {
        let mut results: Vec<hast::Node<'a>> = vec![];
        let mut stack: Vec<Frame<'_, 'a>> = vec![];
        self.enter(node, false, &mut results, &mut stack);
        while let Some(frame) = stack.last_mut() {
            let parent = frame.node;
            let children = parent.children().map_or(&[][..], |children| &children[..]);
            if frame.next < children.len() {
                let index = frame.next;
                frame.next += 1;
                // Text right after a break loses its leading spaces
                let trim =
                    index > 0 && matches!(children[index - 1], mdast::Node::Break(_));
                self.enter(&children[index], trim, &mut results, &mut stack);
                continue;
            }
            let Some(frame) = stack.pop() else {
                break;
            };
            let mut converted =
                ArenaVec::with_capacity_in(results.len() - frame.start, self.alloc);
            converted.extend(results.drain(frame.start..));
            let start = results.len();
            let parent = stack.last().map(|parent| parent.node);
            results.push(self.exit(frame.node, parent, converted));
            if frame.trim {
                trim_start(&mut results[start]);
            }
        }
        let mut nodes = ArenaVec::with_capacity_in(results.len(), self.alloc);
        nodes.extend(results);
        nodes
    }

    fn enter<'n>(
        &mut self,
        node: &'n mdast::Node<'a>,
        trim: bool,
        results: &mut Vec<hast::Node<'a>>,
        stack: &mut Vec<Frame<'n, 'a>>,
    ) {
        let start = results.len();
        if let mdast::Node::List(list) = node {
            self.loose_lists.push(is_list_loose(list));
        }
        if node.children().is_some() {
            stack.push(Frame {
                node,
                next: 0,
                start,
                trim,
            });
            return;
        }
        match node {
            mdast::Node::Text(text) => results.push(convert_text(self, text)),
            mdast::Node::Code(node) => results.push(convert_code(self, node)),
            mdast::Node::InlineCode(node) => {
                results.push(convert_inline_code(self, node))
            }
            mdast::Node::ThematicBreak(node) => {
                results.push(convert_thematic_break(self, node))
            }
            mdast::Node::Break(node) => {
                results.push(convert_hard_break(self, node));
                results.push(line_ending());
            }
            _ => {}
        }
        if trim && let Some(first) = results.get_mut(start) {
            trim_start(first);
        }
    }

    fn exit(
        &mut self,
        node: &mdast::Node<'a>,
        parent: Option<&mdast::Node<'a>>,
        children: ArenaVec<'a, hast::Node<'a>>,
    ) -> hast::Node<'a> {
        match node {
            mdast::Node::Blockquote(node) => convert_blockquote(self, node, children),
            mdast::Node::List(list) => {
                self.loose_lists.pop();
                convert_list(self, list, children)
            }
            mdast::Node::ListItem(item) => {
                let loose = match parent {
                    Some(mdast::Node::List(_)) => self.loose_lists.last().copied(),
                    _ => None,
                };
                convert_list_item(self, item, loose, children)
            }
            mdast::Node::Root(root) => convert_root(self, root, children),
            mdast::Node::Heading(heading) => convert_heading(self, heading, children),
            mdast::Node::Emphasis(emphasis) => convert_emphasis(self, emphasis, children),
            mdast::Node::Strong(strong) => convert_strong(self, strong, children),
            mdast::Node::Paragraph(paragraph) => {
                convert_paragraph(self, paragraph, children)
            }
            mdast::Node::ThematicBreak(_)
            | mdast::Node::Text(_)
            | mdast::Node::Break(_)
            | mdast::Node::Code(_)
            | mdast::Node::InlineCode(_) => {
                unreachable!("leaves convert on enter")
            }
        }
    }

    /// Put a line ending between nodes, and around them when `loose`.
    pub(crate) fn wrap(
        &self,
        nodes: ArenaVec<'a, hast::Node<'a>>,
        loose: bool,
    ) -> ArenaVec<'a, hast::Node<'a>> {
        let mut result = ArenaVec::with_capacity_in(nodes.len() * 2 + 1, self.alloc);
        let any = !nodes.is_empty();
        if loose {
            result.push(line_ending());
        }
        for (index, node) in nodes.into_iter().enumerate() {
            if index > 0 {
                result.push(line_ending());
            }
            result.push(node);
        }
        if loose && any {
            result.push(line_ending());
        }
        result
    }

    /// An element with no properties.
    pub(crate) fn element(
        &self,
        tag_name: &'a str,
        children: ArenaVec<'a, hast::Node<'a>>,
        position: Option<Position<'a>>,
    ) -> hast::Node<'a> {
        hast::Node::Element(Element {
            tag_name,
            properties: ArenaVec::new_in(self.alloc),
            children,
            position,
        })
    }
}

// As trimMarkdownSpaceStart does, on a text node or the first child of an element
fn trim_start(node: &mut hast::Node<'_>) {
    let text = match node {
        hast::Node::Text(text) => Some(text),
        hast::Node::Element(element) => match element.children.first_mut() {
            Some(hast::Node::Text(text)) => Some(text),
            _ => None,
        },
        _ => None,
    };
    if let Some(text) = text {
        text.value = text.value.trim_start_matches([' ', '\t']);
    }
}

fn line_ending<'a>() -> hast::Node<'a> {
    hast::Node::Text(hast::Text {
        value: "\n",
        position: None,
    })
}
