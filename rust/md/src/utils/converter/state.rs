use std::collections::HashMap;

use textlane_core::hast::{self, Element, Property, PropertyValue};
use textlane_core::mdast;
use textlane_core::{Allocator, ArenaString, ArenaVec, Position};

use crate::actions::blockquote::convert::convert_blockquote;
use crate::actions::code::convert::convert_code;
use crate::actions::def_list::convert::convert_def_list;
use crate::actions::def_list_description::convert::convert_def_list_description;
use crate::actions::def_list_term::convert::convert_def_list_term;
use crate::actions::delete::convert::convert_delete;
use crate::actions::emphasis::convert::convert_emphasis;
use crate::actions::footnote_reference::convert::convert_footnote_reference;
use crate::actions::hard_break::convert::convert_hard_break;
use crate::actions::heading::convert::convert_heading;
use crate::actions::html::convert::convert_html;
use crate::actions::image::convert::convert_image;
use crate::actions::image_reference::convert::convert_image_reference;
use crate::actions::inline_code::convert::convert_inline_code;
use crate::actions::link::convert::convert_link;
use crate::actions::link_reference::convert::convert_link_reference;
use crate::actions::list::convert::convert_list;
use crate::actions::list_item::convert::{convert_list_item, is_list_loose};
use crate::actions::mdx_expression::convert::convert_mdx_expression;
use crate::actions::mdx_jsx_element::convert::convert_mdx_jsx_element;
use crate::actions::paragraph::convert::convert_paragraph;
use crate::actions::root::convert::convert_root;
use crate::actions::strong::convert::convert_strong;
use crate::actions::table::convert::convert_table;
use crate::actions::table_cell::convert::convert_table_cell;
use crate::actions::table_row::convert::convert_table_row;
use crate::actions::text::convert::convert_text;
use crate::actions::thematic_break::convert::convert_thematic_break;
use crate::models::options::ToHtml;
use crate::utils::converter::unknown::{apply_data, convert_unknown};

/// Turns mdast into hast, as mdast-util-to-hast does.
pub(crate) struct Converter<'a, 'o> {
    pub(crate) alloc: &'a Allocator,
    pub(crate) options: &'o ToHtml,
    /// Whether paragraphs of only MDX JSX, expressions and white space unravel.
    pub(crate) unravel: bool,
    /// Whether each list being converted is loose, worked out once per list for its items.
    pub(crate) loose_lists: Vec<bool>,
    /// The URL and title of each definition, by its identifier in upper case; the first
    /// definition of an identifier wins.
    pub(crate) definitions: HashMap<String, (&'a str, Option<&'a str>)>,
    /// The identifiers of called footnotes in upper case, in the order of their first call.
    pub(crate) footnote_order: Vec<String>,
    /// For each called footnote, where it is in `footnote_order` and how often it is called.
    pub(crate) footnote_counts: HashMap<String, (usize, usize)>,
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
            let count = results.len() - frame.start;
            // Wrapped children gain a line ending between and around them, in place
            let capacity = match frame.node {
                mdast::Node::Root(_)
                | mdast::Node::Blockquote(_)
                | mdast::Node::List(_) => count * 2 + 1,
                _ => count,
            };
            let mut converted = ArenaVec::with_capacity_in(capacity, self.alloc);
            converted.extend(results.drain(frame.start..));
            let start = results.len();
            let parent = stack.last().map(|parent| parent.node);
            self.exit(frame.node, parent, converted, &mut results);
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
        // Footnote definitions turn up in the footer only, once called
        if matches!(node, mdast::Node::FootnoteDefinition(_)) {
            return;
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
        // Whether the handler applies `data`: `code` does on its inner element itself, and
        // the source text of an undefined reference takes none
        let (result, applies) = match node {
            mdast::Node::Text(text) => (convert_text(self, text), true),
            mdast::Node::Code(node) => (convert_code(self, node), false),
            mdast::Node::InlineCode(node) => (convert_inline_code(self, node), true),
            mdast::Node::ThematicBreak(node) => {
                (convert_thematic_break(self, node), true)
            }
            mdast::Node::Break(node) => (convert_hard_break(self, node), true),
            mdast::Node::Image(node) => (convert_image(self, node), true),
            mdast::Node::Html(node) => match convert_html(self, node) {
                Some(result) => (result, true),
                None => return,
            },
            mdast::Node::ImageReference(node) => (
                convert_image_reference(self, node),
                self.find_definition(node.identifier).is_some(),
            ),
            mdast::Node::FootnoteReference(node) => {
                (convert_footnote_reference(self, node), true)
            }
            mdast::Node::MdxFlowExpression(_)
            | mdast::Node::MdxTextExpression(_)
            | mdast::Node::MdxjsEsm(_) => {
                results.extend(convert_mdx_expression(node));
                return;
            }
            mdast::Node::Math(_)
            | mdast::Node::InlineMath(_)
            | mdast::Node::Literal(_)
            | mdast::Node::Unknown(_) => (
                convert_unknown(self, node, ArenaVec::new_in(self.alloc)),
                false,
            ),
            _ => return,
        };
        let result = if applies {
            apply_data(self, node.data(), result)
        } else {
            result
        };
        results.push(result);
        if matches!(node, mdast::Node::Break(_)) {
            results.push(line_ending());
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
        results: &mut Vec<hast::Node<'a>>,
    ) {
        let result = match node {
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
            mdast::Node::Delete(delete) => convert_delete(self, delete, children),
            mdast::Node::Emphasis(emphasis) => convert_emphasis(self, emphasis, children),
            mdast::Node::Strong(strong) => convert_strong(self, strong, children),
            mdast::Node::Paragraph(paragraph) => {
                if self.unravel && is_unravelable(&paragraph.children) {
                    unravel_paragraph(&paragraph.children, children, results);
                    return;
                }
                convert_paragraph(self, paragraph, children)
            }
            mdast::Node::Link(link) => convert_link(self, link, children),
            mdast::Node::Table(table) => convert_table(self, table, children),
            mdast::Node::TableRow(row) => {
                convert_table_row(self, row, node, parent, children)
            }
            // A row takes its cells' `data` itself
            mdast::Node::TableCell(cell) => {
                let result = convert_table_cell(self, cell, children);
                if matches!(parent, Some(mdast::Node::TableRow(_))) {
                    results.push(result);
                    return;
                }
                result
            }
            // remark-definition-list's handlers apply no `data`
            mdast::Node::DefList(list) => {
                results.push(convert_def_list(self, list, children));
                return;
            }
            mdast::Node::DefListTerm(term) => {
                results.push(convert_def_list_term(self, term, children));
                return;
            }
            mdast::Node::DefListDescription(description) => {
                results.push(convert_def_list_description(self, description, children));
                return;
            }
            mdast::Node::LinkReference(reference) => {
                convert_link_reference(self, reference, children, results);
                // The source text of an undefined reference takes no `data`
                if self.find_definition(reference.identifier).is_some()
                    && let Some(result) = results.pop()
                {
                    results.push(apply_data(self, node.data(), result));
                }
                return;
            }
            mdast::Node::MdxJsxFlowElement(_) | mdast::Node::MdxJsxTextElement(_) => {
                results.extend(convert_mdx_jsx_element(node, children));
                return;
            }
            mdast::Node::ContainerDirective(_)
            | mdast::Node::LeafDirective(_)
            | mdast::Node::TextDirective(_)
            | mdast::Node::Unknown(_) => {
                results.push(convert_unknown(self, node, children));
                return;
            }
            _ => return,
        };
        results.push(apply_data(self, node.data(), result));
    }

    /// The URL and title of the definition an identifier refers to.
    pub(crate) fn find_definition(
        &self,
        identifier: &str,
    ) -> Option<(&'a str, Option<&'a str>)> {
        self.definitions.get(&identifier.to_uppercase()).copied()
    }

    /// What footnote `id`s start with.
    pub(crate) fn clobber_prefix(&self) -> &'a str {
        let prefix = self
            .options
            .clobber_prefix
            .as_deref()
            .unwrap_or("user-content-");
        self.alloc.alloc_str(prefix)
    }

    /// Add a property when it has a value.
    pub(crate) fn push_property(
        &self,
        properties: &mut ArenaVec<'a, Property<'a>>,
        name: &'a str,
        value: Option<&'a str>,
    ) {
        if let Some(value) = value {
            properties.push(Property {
                name,
                value: PropertyValue::String(value),
            });
        }
    }

    /// A text node.
    pub(crate) fn text(&self, value: &'a str) -> hast::Node<'a> {
        hast::Node::Text(hast::Text {
            value,
            position: None,
            data: None,
        })
    }

    /// Strings joined in the arena.
    pub(crate) fn concat(&self, pieces: &[&str]) -> &'a str {
        let mut value = ArenaString::new_in(self.alloc);
        pieces.iter().for_each(|piece| value.push_str(piece));
        value.into_bump_str()
    }

    /// An element with properties.
    pub(crate) fn element_with(
        &self,
        tag_name: &'a str,
        properties: ArenaVec<'a, Property<'a>>,
        children: ArenaVec<'a, hast::Node<'a>>,
        position: Option<Position<'a>>,
    ) -> hast::Node<'a> {
        hast::Node::Element(Element {
            tag_name,
            properties,
            children,
            position,
            content: None,
            data: None,
        })
    }

    /// Put a line ending between nodes, and around them when `loose`.
    pub(crate) fn wrap(
        &self,
        mut nodes: ArenaVec<'a, hast::Node<'a>>,
        loose: bool,
    ) -> ArenaVec<'a, hast::Node<'a>> {
        let count = nodes.len();
        let outer = usize::from(loose);
        let len = match count {
            0 => outer,
            _ => count * 2 - 1 + outer * 2,
        };
        if nodes.capacity() >= len {
            while nodes.len() < len {
                nodes.push(line_ending());
            }
            // From the back, each node swaps with a line ending already in its place
            for index in (0..count).rev() {
                nodes.swap(index, outer + index * 2);
            }
            return nodes;
        }
        let mut result = ArenaVec::with_capacity_in(len, self.alloc);
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
            content: None,
            data: None,
        })
    }
}

// As trimMarkdownSpaceStart does, on a text node or the first child of an element
// remark-mark-and-unravel: some MDX, and otherwise only text of HTML white space
fn is_unravelable(children: &[mdast::Node<'_>]) -> bool {
    let mut any = false;
    for child in children {
        match child {
            mdast::Node::MdxJsxTextElement(_) | mdast::Node::MdxTextExpression(_) => {
                any = true
            }
            mdast::Node::Text(text)
                if text.value.bytes().all(|byte| {
                    matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b'\x0c')
                }) => {}
            _ => return false,
        }
    }
    any
}

// The MDX becomes flow and white space of tabs, line endings and spaces goes; each child
// converted into exactly one node, so both lists line up
fn unravel_paragraph<'a>(
    mdast_children: &[mdast::Node<'_>],
    children: ArenaVec<'a, hast::Node<'a>>,
    results: &mut Vec<hast::Node<'a>>,
) {
    for (source, child) in mdast_children.iter().zip(children) {
        if let mdast::Node::Text(text) = source
            && !text.value.is_empty()
            && text
                .value
                .bytes()
                .all(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r'))
        {
            continue;
        }
        results.push(match child {
            hast::Node::MdxJsxTextElement(element) => {
                hast::Node::MdxJsxFlowElement(element)
            }
            hast::Node::MdxTextExpression(expression) => {
                hast::Node::MdxFlowExpression(expression)
            }
            child => child,
        });
    }
}

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
        data: None,
    })
}
