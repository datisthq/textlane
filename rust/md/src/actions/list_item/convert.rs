use textlane_core::ArenaVec;
use textlane_core::hast;
use textlane_core::mdast::{self, ListItem};

use crate::utils::converter::state::Converter;

/// A list item becomes `li`; in a tight list its paragraphs unwrap into it. `list_loose`
/// is whether its list is loose, when it is in one.
pub(crate) fn convert_list_item<'a>(
    converter: &mut Converter<'a, '_>,
    node: &ListItem<'a>,
    list_loose: Option<bool>,
    results: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    let loose = list_loose.unwrap_or_else(|| is_item_loose(node));
    let tail_is_paragraph = results.last().is_some_and(is_paragraph);
    let any = !results.is_empty();
    let mut children = ArenaVec::with_capacity_in(results.len() * 2 + 1, converter.alloc);
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
    converter.element("li", children, node.position)
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
    })
}
