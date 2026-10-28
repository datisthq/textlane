use textlane_core::hast::{Node, Property, PropertyValue};
use textlane_core::{Allocator, ArenaVec};

use crate::helpers::slug::Slugger;
use crate::models::options::SlugOptions;

/// Give headings without an `id` one made from their text, as rehype-slug does.
pub fn slug_hast<'a>(node: &mut Node<'a>, options: &SlugOptions, alloc: &'a Allocator) {
    let mut slugger = Slugger::default();
    let mut stack: Vec<&mut Node<'a>> = vec![node];
    while let Some(node) = stack.pop() {
        let children = match node {
            Node::Root(root) => &mut root.children,
            Node::Element(element) => {
                if is_heading(element.tag_name) && !has_id(&element.properties) {
                    let text = collect_text(&element.children);
                    let id = format!("{}{}", options.prefix, slugger.slug(&text));
                    element.properties.push(Property {
                        name: "id",
                        value: PropertyValue::String(alloc.alloc_str(&id)),
                    });
                }
                &mut element.children
            }
            _ => continue,
        };
        // Visited in document order, which decides which repeat gets `-1`
        stack.extend(children.iter_mut().rev());
    }
}

fn is_heading(tag_name: &str) -> bool {
    matches!(tag_name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
}

// rehype-slug checks `!properties.id`, so an empty or false id counts as none
fn has_id(properties: &ArenaVec<'_, Property<'_>>) -> bool {
    properties
        .iter()
        .rev()
        .find(|property| property.name == "id")
        .is_some_and(|property| match &property.value {
            PropertyValue::Null | PropertyValue::Boolean(false) => false,
            PropertyValue::String(value) => !value.is_empty(),
            PropertyValue::Number(value) => *value != 0.0 && !value.is_nan(),
            _ => true,
        })
}

// hast-util-to-string: the text of every text node inside
fn collect_text(children: &[Node<'_>]) -> String {
    let mut text = String::new();
    let mut stack: Vec<&Node> = children.iter().rev().collect();
    while let Some(node) = stack.pop() {
        match node {
            Node::Text(node) => text.push_str(node.value),
            Node::Element(element) => stack.extend(element.children.iter().rev()),
            Node::Root(root) => stack.extend(root.children.iter().rev()),
            _ => {}
        }
    }
    text
}
