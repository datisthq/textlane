use textlane_core::hast::{Node, Text};
use textlane_core::{Allocator, ArenaVec};

use crate::actions::attribute::read::read_attribute;
use crate::models::options::PROSE_IGNORE;

/// Replace each text node outside the ignored elements with what `replace` makes of it,
/// when it makes anything; a `template`'s content is never reached.
pub(crate) fn replace_prose<'a>(
    node: &mut Node<'a>,
    ignore: &Option<Vec<String>>,
    ignore_attributes: &[String],
    alloc: &'a Allocator,
    mut replace: impl FnMut(&str, &mut ArenaVec<'a, Node<'a>>) -> bool,
) {
    let ignored = |tag_name: &str| match ignore {
        Some(names) => names.iter().any(|name| name.eq_ignore_ascii_case(tag_name)),
        None => PROSE_IGNORE
            .iter()
            .any(|name| name.eq_ignore_ascii_case(tag_name)),
    };
    let mut stack: Vec<&mut Node<'a>> = vec![node];
    while let Some(node) = stack.pop() {
        let children = match node {
            Node::Root(root) => &mut root.children,
            Node::Element(element) => {
                let skipped = ignored(element.tag_name)
                    || (ignore_attributes.iter())
                        .any(|name| read_attribute(element, name).is_some());
                if skipped {
                    continue;
                }
                &mut element.children
            }
            _ => continue,
        };
        if children.iter().any(|child| matches!(child, Node::Text(_))) {
            let old = std::mem::replace(children, ArenaVec::new_in(alloc));
            for child in old {
                match child {
                    Node::Text(text) => {
                        let start = children.len();
                        if !replace(text.value, children) {
                            children.truncate(start);
                            children.push(Node::Text(text));
                        }
                    }
                    child => children.push(child),
                }
            }
        }
        stack.extend(children.iter_mut());
    }
}

/// A text node holding `value`.
pub(crate) fn create_text<'a>(value: &str, alloc: &'a Allocator) -> Node<'a> {
    Node::Text(Text {
        value: alloc.alloc_str(value),
        position: None,
        data: None,
    })
}
