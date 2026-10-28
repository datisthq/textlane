use textlane_core::hast::{self, Element};
use textlane_core::mdast;
use textlane_core::{Allocator, ArenaVec};

use crate::actions::heading::convert::convert_heading;
use crate::actions::paragraph::convert::convert_paragraph;
use crate::actions::root::convert::convert_root;
use crate::actions::text::convert::convert_text;
use crate::actions::thematic_break::convert::convert_thematic_break;
use crate::models::options::ToHtmlOptions;

/// Turns mdast into hast, as mdast-util-to-hast does.
pub(crate) struct Converter<'a, 'o> {
    pub(crate) alloc: &'a Allocator,
    // `allowDangerousHtml` decides what raw HTML becomes, once html nodes exist
    #[allow(dead_code)]
    pub(crate) options: &'o ToHtmlOptions,
}

impl<'a> Converter<'a, '_> {
    /// Convert one node, or nothing when it has no HTML.
    pub(crate) fn one(&mut self, node: &mdast::Node<'a>) -> Option<hast::Node<'a>> {
        match node {
            mdast::Node::Root(root) => Some(convert_root(self, root)),
            mdast::Node::Paragraph(paragraph) => Some(convert_paragraph(self, paragraph)),
            mdast::Node::Heading(heading) => Some(convert_heading(self, heading)),
            mdast::Node::ThematicBreak(node) => Some(convert_thematic_break(self, node)),
            mdast::Node::Text(text) => Some(convert_text(self, text)),
        }
    }

    /// Convert every child of a parent.
    pub(crate) fn all(
        &mut self,
        children: &[mdast::Node<'a>],
    ) -> ArenaVec<'a, hast::Node<'a>> {
        let mut values = ArenaVec::new_in(self.alloc);
        for child in children {
            if let Some(value) = self.one(child) {
                values.push(value);
            }
        }
        values
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
        position: Option<textlane_core::Position<'a>>,
    ) -> hast::Node<'a> {
        hast::Node::Element(Element {
            tag_name,
            properties: ArenaVec::new_in(self.alloc),
            children,
            position,
        })
    }
}

fn line_ending<'a>() -> hast::Node<'a> {
    hast::Node::Text(hast::Text {
        value: "\n",
        position: None,
    })
}
