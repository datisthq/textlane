use textlane_core::hast::{Element, Node};

use crate::models::visit::Visit;

/// Call `visitor` on every element under `node`, in document order, with the tag names of
/// its element ancestors, outermost first. A `template`'s content is not visited.
pub fn visit_elements<'a>(
    node: &mut Node<'a>,
    mut visitor: impl FnMut(&mut Element<'a>, &[&'a str]) -> Visit,
) {
    let mut ancestors: Vec<&'a str> = vec![];
    // Explicit, as a tree nests as deep as its input does
    let mut stack: Vec<(&mut Node<'a>, usize)> = vec![(node, 0)];
    while let Some((node, depth)) = stack.pop() {
        ancestors.truncate(depth);
        let children = match node {
            Node::Root(root) => &mut root.children,
            Node::Element(element) => {
                match visitor(element, &ancestors) {
                    Visit::Stop => return,
                    Visit::Skip => continue,
                    Visit::Continue => {}
                }
                ancestors.push(element.tag_name);
                &mut element.children
            }
            _ => continue,
        };
        let depth = ancestors.len();
        for child in children.iter_mut().rev() {
            stack.push((child, depth));
        }
    }
}

#[cfg(test)]
#[path = "visit_unit.rs"]
mod unit;
