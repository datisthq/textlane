use textlane_core::Error;
use textlane_core::mdast::{ListItem, Node};

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::settings::ListItemIndent;
use crate::utils::serializer::sink::Layer;
use crate::utils::serializer::state::{Parent, Serializer};

/// Write a list item: its bullet, then its content indented to line up after it.
pub(crate) fn stringify_list_item(
    state: &mut Serializer,
    item: &ListItem,
    node: Parent,
    parent: Option<Parent>,
) -> Result<(), Error> {
    let settings = state.settings;
    let mut bullet = state
        .bullet_current
        .clone()
        .unwrap_or_else(|| settings.bullet.to_string());
    let list = match parent.and_then(|parent| parent.node) {
        Some(Node::List(list)) => Some(list),
        _ => None,
    };
    if let Some(list) = list.filter(|list| list.ordered) {
        let index = state.index_stack.last().copied().unwrap_or_default();
        let start = u64::from(list.start.unwrap_or(1));
        let offset = if settings.increment_list_marker {
            index as u64
        } else {
            0
        };
        bullet = format!("{}{bullet}", start + offset);
    }
    let mut size = bullet.len() + 1;
    let spread =
        list.is_some_and(|list| list.spread == Some(true)) || item.spread == Some(true);
    if settings.list_item_indent == ListItemIndent::Tab
        || (settings.list_item_indent == ListItemIndent::Mixed && spread)
    {
        size = size.div_ceil(4) * 4;
    }
    state.enter(Construct::ListItem);
    state.open_layer(Layer::ListItem { bullet, size });
    let result = state.container_flow(node);
    state.close_layer();
    state.exit();
    result
}
