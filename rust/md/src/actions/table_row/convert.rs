use textlane_core::ArenaVec;
use textlane_core::hast::{self, Property, PropertyValue};
use textlane_core::mdast::{AlignKind, Node, TableRow};

use crate::utils::converter::state::Converter;

/// A row becomes `tr`: `th` cells in the first row of its parent, `td` cells otherwise,
/// one per column of a table, aligned as it says.
pub(crate) fn convert_table_row<'a>(
    converter: &mut Converter<'a, '_>,
    node: &TableRow<'a>,
    this: &Node<'a>,
    parent: Option<&Node<'a>>,
    children: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    let first = parent
        .and_then(Node::children)
        .and_then(|siblings| siblings.first())
        .is_some_and(|first| std::ptr::eq(first, this));
    let tag_name = if parent.is_none() || !first {
        "td"
    } else {
        "th"
    };
    // An empty `align` is read as none: a tree from JSON without one has it empty
    let align = match parent {
        Some(Node::Table(table)) if !table.align.is_empty() => Some(&table.align),
        _ => None,
    };
    let length = align.map_or(node.children.len(), |align| align.len());
    let mut cells = ArenaVec::with_capacity_in(length, converter.alloc);
    let mut converted = children.into_iter();
    for index in 0..length {
        let mut properties = ArenaVec::new_in(converter.alloc);
        let value = match align.and_then(|align| align.get(index)) {
            Some(AlignKind::Left) => Some("left"),
            Some(AlignKind::Right) => Some("right"),
            Some(AlignKind::Center) => Some("center"),
            _ => None,
        };
        if let Some(value) = value {
            properties.push(Property {
                name: "align",
                value: PropertyValue::String(value),
            });
        }
        let cell = match converted.next() {
            Some(hast::Node::Element(cell)) => {
                converter.element_with(tag_name, properties, cell.children, cell.position)
            }
            _ => converter.element_with(
                tag_name,
                properties,
                ArenaVec::new_in(converter.alloc),
                None,
            ),
        };
        cells.push(cell);
    }
    let cells = converter.wrap(cells, true);
    converter.element("tr", cells, node.position)
}
