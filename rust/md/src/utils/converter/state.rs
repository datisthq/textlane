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
use crate::actions::inline_math::convert::convert_inline_math;
use crate::actions::link::convert::convert_link;
use crate::actions::link_reference::convert::convert_link_reference;
use crate::actions::list::convert::convert_list;
use crate::actions::list_item::convert::{convert_list_item, is_list_loose};
use crate::actions::literal::convert::convert_literal;
use crate::actions::math::convert::convert_math;
use crate::actions::paragraph::convert::convert_paragraph;
use crate::actions::root::convert::convert_root;
use crate::actions::strong::convert::convert_strong;
use crate::actions::table::convert::convert_table;
use crate::actions::table_cell::convert::convert_table_cell;
use crate::actions::table_row::convert::convert_table_row;
use crate::actions::text::convert::convert_text;
use crate::actions::thematic_break::convert::convert_thematic_break;
use crate::models::options::ToHtmlOptions;

/// Turns mdast into hast, as mdast-util-to-hast does.
pub(crate) struct Converter<'a, 'o> {
    pub(crate) alloc: &'a Allocator,
    pub(crate) options: &'o ToHtmlOptions,
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
            let mut converted =
                ArenaVec::with_capacity_in(results.len() - frame.start, self.alloc);
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
            mdast::Node::Image(node) => results.push(convert_image(self, node)),
            mdast::Node::Html(node) => results.extend(convert_html(self, node)),
            mdast::Node::ImageReference(node) => {
                results.push(convert_image_reference(self, node));
            }
            mdast::Node::FootnoteReference(node) => {
                results.push(convert_footnote_reference(self, node));
            }
            mdast::Node::Math(node) => results.push(convert_math(self, node)),
            mdast::Node::InlineMath(node) => {
                results.push(convert_inline_math(self, node))
            }
            mdast::Node::Literal(node) => results.push(convert_literal(node)),
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
                convert_paragraph(self, paragraph, children)
            }
            mdast::Node::Link(link) => convert_link(self, link, children),
            mdast::Node::Table(table) => convert_table(self, table, children),
            mdast::Node::TableRow(row) => {
                convert_table_row(self, row, node, parent, children)
            }
            mdast::Node::TableCell(cell) => convert_table_cell(self, cell, children),
            mdast::Node::DefList(list) => convert_def_list(self, list, children),
            mdast::Node::DefListTerm(term) => convert_def_list_term(self, term, children),
            mdast::Node::DefListDescription(description) => {
                convert_def_list_description(self, description, children)
            }
            mdast::Node::LinkReference(reference) => {
                convert_link_reference(self, reference, children, results);
                return;
            }
            _ => return,
        };
        results.push(result);
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
            content: None,
            data: None,
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
        data: None,
    })
}
