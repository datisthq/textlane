use textlane_core::ArenaVec;
use textlane_core::hast::{self, Element, Property, PropertyItem, PropertyValue};
use textlane_core::mdast::{self, ListItem};

use crate::utils::converter::state::Converter;

/// A list item becomes `li`; in a tight list its paragraphs unwrap into it. `list_loose`
/// is whether its list is loose, when it is in one. A task item starts with a checkbox.
pub(crate) fn convert_list_item<'a>(
    converter: &mut Converter<'a, '_>,
    node: &ListItem<'a>,
    list_loose: Option<bool>,
    mut results: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    let mut properties = ArenaVec::new_in(converter.alloc);
    if let Some(checked) = node.checked {
        add_checkbox(converter, &mut results, checked);
        properties.push(create_class_name(converter, "task-list-item"));
    }
    let loose = list_loose.unwrap_or_else(|| is_item_loose(node));
    let tail_is_paragraph = results.last().is_some_and(is_paragraph);
    let any = !results.is_empty();
    let unwrapped: usize = results
        .iter()
        .map(|child| match child {
            hast::Node::Element(paragraph) if paragraph.tag_name == "p" && !loose => {
                paragraph.children.len()
            }
            _ => 1,
        })
        .sum();
    let mut children =
        ArenaVec::with_capacity_in(unwrapped + results.len() + 1, converter.alloc);
    for (index, child) in results.into_iter().enumerate() {
        // A line ending before each child, except a tight item's first paragraph
        if loose || index != 0 || !is_paragraph(&child) {
            children.push(line_ending());
        }
        match child {
            hast::Node::Element(paragraph) if paragraph.tag_name == "p" && !loose => {
                children.extend(paragraph.children);
            }
            child => children.push(child),
        }
    }
    if any && (loose || !tail_is_paragraph) {
        children.push(line_ending());
    }
    converter.element_with("li", properties, children, node.position)
}

// Into the first paragraph, or a new one before the rest
fn add_checkbox<'a>(
    converter: &Converter<'a, '_>,
    results: &mut ArenaVec<'a, hast::Node<'a>>,
    checked: bool,
) {
    if !results.first().is_some_and(is_paragraph) {
        let paragraph = converter.element("p", ArenaVec::new_in(converter.alloc), None);
        results.insert(0, paragraph);
    }
    let Some(hast::Node::Element(paragraph)) = results.first_mut() else {
        return;
    };
    if !paragraph.children.is_empty() {
        paragraph.children.insert(0, converter.text(" "));
    }
    let mut properties = ArenaVec::with_capacity_in(3, converter.alloc);
    properties.push(Property {
        name: "type",
        value: PropertyValue::String("checkbox"),
    });
    properties.push(Property {
        name: "checked",
        value: PropertyValue::Boolean(checked),
    });
    properties.push(Property {
        name: "disabled",
        value: PropertyValue::Boolean(true),
    });
    let input = hast::Node::Element(Element {
        tag_name: "input",
        properties,
        children: ArenaVec::new_in(converter.alloc),
        position: None,
        content: None,
        data: None,
    });
    paragraph.children.insert(0, input);
}

/// A `className` property of one class.
pub(crate) fn create_class_name<'a>(
    converter: &Converter<'a, '_>,
    name: &'a str,
) -> Property<'a> {
    let mut list = ArenaVec::with_capacity_in(1, converter.alloc);
    list.push(PropertyItem::String(name));
    Property {
        name: "className",
        value: PropertyValue::List(list),
    }
}

/// As mdast-util-to-hast: a list is loose when it or any of its items is.
pub(crate) fn is_list_loose(list: &mdast::List<'_>) -> bool {
    list.spread.unwrap_or(false)
        || list.children.iter().any(|child| match child {
            mdast::Node::ListItem(item) => is_item_loose(item),
            _ => false,
        })
}

fn is_item_loose(item: &ListItem<'_>) -> bool {
    item.spread.unwrap_or(item.children.len() > 1)
}

fn is_paragraph(node: &hast::Node<'_>) -> bool {
    matches!(node, hast::Node::Element(element) if element.tag_name == "p")
}

fn line_ending<'a>() -> hast::Node<'a> {
    hast::Node::Text(hast::Text {
        value: "\n",
        position: None,
        data: None,
    })
}
