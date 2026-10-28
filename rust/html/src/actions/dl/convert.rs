use textlane_core::hast::{Element, Node};

use crate::utils::converter::state::{Converter, push_converted};
use crate::utils::converter::store::{Converted, Id, Kind};

/// `<dl>`: a list with an item per group of terms and their descriptions.
pub(crate) fn convert_dl<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    element: &'n Element<'a>,
) -> Converted {
    let mut clean: Vec<&'n Node<'a>> = vec![];
    for child in &element.children {
        match child {
            Node::Element(div) if div.tag_name == "div" => {
                clean.extend(div.children.iter())
            }
            _ => clean.push(child),
        }
    }
    let mut groups = vec![];
    let mut group: (Vec<&'n Node<'a>>, Vec<&'n Node<'a>>) = (vec![], vec![]);
    for (index, child) in clean.iter().enumerate() {
        if is_element(child, "dt") {
            if index > 0 && is_element(clean[index - 1], "dd") {
                groups.push(std::mem::take(&mut group));
            }
            group.0.push(child);
        } else {
            group.1.push(child);
        }
    }
    groups.push(group);
    let mut content = vec![];
    for (titles, definitions) in groups {
        let mut result = handle(state, &titles);
        result.extend(handle(state, &definitions));
        if !result.is_empty() {
            let kind = Kind::ListItem {
                spread: result.len() > 1,
                checked: None,
            };
            content.push(state.store.add(kind, result, None));
        }
    }
    if content.is_empty() {
        return Converted::Nothing;
    }
    let kind = Kind::List {
        ordered: false,
        start: None,
        spread: state.store.is_spread(&content),
    };
    Converted::One(state.store.add(kind, content, element.position))
}

fn handle<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    children: &[&'n Node<'a>],
) -> Vec<Id> {
    let mut nodes = vec![];
    for child in children {
        let converted = state.one(child);
        push_converted(&mut nodes, converted);
    }
    let items = state.wrap_specific_content(nodes, Kind::list_item());
    match items[..] {
        [] => vec![],
        [item] => {
            let children = state.store.take_children(item);
            state.store.expand(&children).into_owned()
        }
        _ => {
            let kind = Kind::List {
                ordered: false,
                start: None,
                spread: state.store.is_spread(&items),
            };
            vec![state.store.add(kind, items, None)]
        }
    }
}

fn is_element(node: &Node<'_>, tag_name: &str) -> bool {
    matches!(node, Node::Element(element) if element.tag_name == tag_name)
}
