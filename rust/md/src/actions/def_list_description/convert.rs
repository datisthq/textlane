use textlane_core::ArenaVec;
use textlane_core::hast;
use textlane_core::mdast::DefListDescription;

use crate::utils::converter::state::Converter;

/// A definition description becomes `dd`; unless it is spread, its paragraphs unwrap into
/// it, as list items' do in a tight list.
pub(crate) fn convert_def_list_description<'a>(
    converter: &mut Converter<'a, '_>,
    node: &DefListDescription<'a>,
    results: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    let spread = node.spread;
    let mut children = ArenaVec::with_capacity_in(results.len() * 2 + 1, converter.alloc);
    for (index, child) in results.into_iter().enumerate() {
        if spread || index != 0 || !is_paragraph(&child) {
            children.push(converter.text("\n"));
        }
        match child {
            hast::Node::Element(paragraph) if paragraph.tag_name == "p" && !spread => {
                children.extend(paragraph.children);
            }
            child => children.push(child),
        }
    }
    // After unwrapping, the last child is a paragraph only when spread
    if children.last().is_some_and(|tail| {
        spread || !matches!(tail, hast::Node::Element(element) if element.tag_name == "p")
    }) {
        children.push(converter.text("\n"));
    }
    converter.element("dd", children, node.position)
}

fn is_paragraph(node: &hast::Node<'_>) -> bool {
    matches!(node, hast::Node::Element(element) if element.tag_name == "p")
}
