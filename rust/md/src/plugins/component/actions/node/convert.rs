use std::collections::BTreeMap;

use textlane_core::mdast::Node;
use textlane_core::{Allocator, ArenaVec};

use crate::plugins::component::actions::element::create::create_element;

/// The component `nodes` maps a parent node's type to, keeping its children and position
/// but no other field; any other node comes back as it was.
pub(crate) fn convert_node<'a>(
    nodes: &BTreeMap<String, String>,
    mut node: Node<'a>,
    alloc: &'a Allocator,
) -> Node<'a> {
    let Some(component) = read_type(&node).and_then(|kind| nodes.get(kind)) else {
        return node;
    };
    let text = is_phrasing(&node);
    let position = node.position().copied();
    let children = match node.children_mut() {
        Some(children) => std::mem::replace(children, ArenaVec::new_in(alloc)),
        None => return node,
    };
    let name = alloc.alloc_str(component);
    create_element(name, Vec::new(), children, position, text, alloc)
}

// The mdast type of each parent `nodes` can map; a JSX element or a directive is never one
fn read_type(node: &Node<'_>) -> Option<&'static str> {
    Some(match node {
        Node::Paragraph(_) => "paragraph",
        Node::Heading(_) => "heading",
        Node::Emphasis(_) => "emphasis",
        Node::Strong(_) => "strong",
        Node::Blockquote(_) => "blockquote",
        Node::List(_) => "list",
        Node::ListItem(_) => "listItem",
        Node::Link(_) => "link",
        Node::LinkReference(_) => "linkReference",
        Node::Delete(_) => "delete",
        Node::Table(_) => "table",
        Node::TableRow(_) => "tableRow",
        Node::TableCell(_) => "tableCell",
        Node::FootnoteDefinition(_) => "footnoteDefinition",
        Node::DefList(_) => "defList",
        Node::DefListTerm(_) => "defListTerm",
        Node::DefListDescription(_) => "defListDescription",
        _ => return None,
    })
}

fn is_phrasing(node: &Node<'_>) -> bool {
    matches!(
        node,
        Node::Emphasis(_)
            | Node::Strong(_)
            | Node::Link(_)
            | Node::LinkReference(_)
            | Node::Delete(_)
    )
}
