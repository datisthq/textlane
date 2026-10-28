use std::collections::HashMap;

use textlane_core::hast::{Element, Node};

use crate::actions::p::convert::build_paragraph;
use crate::helpers::phrasing::is_phrasing;
use crate::helpers::value::{find_string, find_value, is_truthy};
use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Id, Kind};

// Spread checks recurse once per level; past the red zone the stack grows on the heap
const STACK_RED_ZONE: usize = 64 * 1024;
const STACK_GROWTH: usize = 1024 * 1024;

/// `<li>`, `<dt>` and `<dd>`: a list item of flow content, checked by a leading checkbox.
pub(crate) fn convert_li<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    element: &'n Element<'a>,
) -> Converted {
    let checkbox = find_leading_checkbox(element);
    let checked =
        checkbox.map(|(checkbox, _)| is_truthy(find_value(checkbox, "checked")));
    // Dropping the checkbox, which is phrasing, never changes whether the item is spread
    let spread = is_spread_out(&mut state.spread, element);
    let children = match checkbox {
        Some((_, depth)) => all_without_checkbox(state, &element.children, depth),
        None => state.all(&element.children),
    };
    let children = state.wrap_flow(children);
    let kind = Kind::ListItem { spread, checked };
    Converted::One(state.store.add(kind, children, element.position))
}

// A checkbox or radio first in the item, or first in a paragraph first in it, and how many
// paragraphs deep
fn find_leading_checkbox<'n, 'a>(
    element: &'n Element<'a>,
) -> Option<(&'n Element<'a>, usize)> {
    let mut parent = element;
    let mut depth = 0;
    loop {
        match parent.children.first() {
            Some(Node::Element(head))
                if head.tag_name == "input"
                    && matches!(
                        find_string(head, "type"),
                        Some("checkbox" | "radio")
                    ) =>
            {
                return Some((head, depth));
            }
            Some(Node::Element(head)) if head.tag_name == "p" => {
                parent = head;
                depth += 1;
            }
            _ => return None,
        }
    }
}

fn all_without_checkbox<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    children: &'n [Node<'a>],
    depth: usize,
) -> Vec<Id> {
    let mut results = vec![];
    if depth > 0
        && let Some(Node::Element(head)) = children.first()
        && find_string(head, "dataMdast") != Some("ignore")
    {
        let nested = stacker::maybe_grow(STACK_RED_ZONE, STACK_GROWTH, || {
            all_without_checkbox(state, &head.children, depth - 1)
        });
        if let Converted::One(paragraph) = build_paragraph(state, head, nested) {
            results.push(paragraph);
        }
    }
    results.extend(state.all(children.get(1..).unwrap_or_default()));
    results
}

// Whether an element holds a paragraph, or two flow elements, at any depth
fn is_spread_out(memo: &mut HashMap<usize, bool>, element: &Element<'_>) -> bool {
    let key = element as *const Element<'_> as usize;
    if let Some(spread) = memo.get(&key) {
        return *spread;
    }
    let mut spread = false;
    let mut seen_flow = false;
    for child in &element.children {
        let Node::Element(child_element) = child else {
            continue;
        };
        if is_phrasing(child) {
            continue;
        }
        if child_element.tag_name == "p"
            || seen_flow
            || stacker::maybe_grow(STACK_RED_ZONE, STACK_GROWTH, || {
                is_spread_out(memo, child_element)
            })
        {
            spread = true;
            break;
        }
        seen_flow = true;
    }
    memo.insert(key, spread);
    spread
}
