use std::collections::HashMap;

use textlane_core::hast::{Element, Node};
use textlane_core::{Allocator, mdast};

use crate::actions::hast::minify::minify_hast;
use crate::helpers::value::read_value;
use crate::models::options::ToMarkdown;
use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind, Store};

/// Turn hast into mdast, as hast-util-to-mdast does after collapsing whitespace as
/// rehype-minify-whitespace does; several nodes come back in a root.
pub fn convert_hast<'a>(
    mut node: Node<'a>,
    options: &ToMarkdown,
    alloc: &'a Allocator,
) -> mdast::Node<'a> {
    minify_hast(&mut node, options.newlines, alloc);
    let mut converter = Converter {
        alloc,
        options,
        store: Store::default(),
        element_by_id: find_elements_by_id(&node),
        base_found: false,
        base: None,
        in_table: false,
        q_nesting: 0,
        spread: HashMap::new(),
        datalists: HashMap::new(),
    };
    let id = match converter.one(&node) {
        Converted::One(id) => id,
        Converted::Nothing => converter.store.add(Kind::Root, vec![], None),
        Converted::Many(ids) => converter.store.add(Kind::Root, ids, None),
    };
    converter.store.build(id, alloc)
}

// The first element with each id, in document order
fn find_elements_by_id<'n, 'a>(node: &'n Node<'a>) -> HashMap<String, &'n Element<'a>> {
    let mut elements = HashMap::new();
    let mut stack = vec![node];
    while let Some(node) = stack.pop() {
        let children = match node {
            Node::Root(root) => &root.children,
            Node::Element(element) => {
                let id = read_value(element, "id");
                if !id.is_empty() {
                    elements.entry(id.into_owned()).or_insert(element);
                }
                &element.children
            }
            _ => continue,
        };
        stack.extend(children.iter().rev());
    }
    elements
}
