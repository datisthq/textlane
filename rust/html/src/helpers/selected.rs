use std::borrow::Cow;

use textlane_core::hast::{Element, Node};

use crate::helpers::text::read_text;
use crate::helpers::value::{find_value, is_truthy, parse_int, read_value, write_value};

// Options nest only as deep as their markup; past the red zone the stack grows on the heap
const STACK_RED_ZONE: usize = 64 * 1024;
const STACK_GROWTH: usize = 1024 * 1024;

/// Each shown option's value, with its label where that differs.
pub(crate) type OptionValues = Vec<(String, Option<String>)>;

/// The values of the options a `<select>` or `<datalist>` shows, each with its label when
/// that differs; `settings` is the element whose `size` and `multiple` apply.
pub(crate) fn find_selected_options<'a>(
    element: &Element<'a>,
    settings: &Element<'a>,
) -> OptionValues {
    let mut options = vec![];
    find_options(&element.children, &mut options);
    let size = read_shown_count(settings);
    let selected: Vec<&Element<'a>> = options
        .iter()
        .copied()
        .filter(|option| is_truthy(find_value(option, "selected")))
        .collect();
    let list = if selected.is_empty() {
        options
    } else {
        selected
    };
    let max = list.len().min(size);
    list[..max]
        .iter()
        .map(|option| {
            let content = read_text(option);
            let label = if content.is_empty() {
                read_value(option, "label").into_owned()
            } else {
                content.clone()
            };
            let value = match read_value(option, "value") {
                Cow::Borrowed("") => content,
                value => value.into_owned(),
            };
            // An empty label reads as none wherever upstream uses one
            let label = (label != value && !label.is_empty()).then_some(label);
            (value, label)
        })
        .collect()
}

/// How many options an element with `size` and `multiple` shows: `Math.min(size, 0) ||`
/// 4 or 1, so a negative size shows none and any other the default.
pub(crate) fn read_shown_count(settings: &Element<'_>) -> usize {
    let size =
        find_value(settings, "size").and_then(|size| parse_int(&write_value(size)));
    match size {
        Some(size) if size < 0.0 => 0,
        _ if is_truthy(find_value(settings, "multiple")) => 4,
        _ => 1,
    }
}

// Each enabled option, after the options inside it
fn find_options<'n, 'a>(children: &'n [Node<'a>], options: &mut Vec<&'n Element<'a>>) {
    for child in children {
        let Node::Element(element) = child else {
            continue;
        };
        stacker::maybe_grow(STACK_RED_ZONE, STACK_GROWTH, || {
            find_options(&element.children, options);
        });
        if element.tag_name == "option" && !is_truthy(find_value(element, "disabled")) {
            options.push(element);
        }
    }
}
