use textlane_core::Error;
use textlane_core::mdast::{List, Node};

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::state::{Parent, Serializer};

/// Write a list, switching to the other bullet where the configured one would merge it
/// into the list before it or read as a thematic break.
pub(crate) fn stringify_list(
    state: &mut Serializer,
    list: &List,
    node: Parent,
    parent: Option<Parent>,
) -> Result<(), Error> {
    state.enter(Construct::List);
    let settings = state.settings;
    let bullet_current = state.bullet_current.take();
    let mut bullet = if list.ordered {
        settings.bullet_ordered
    } else {
        settings.bullet
    };
    let bullet_other = if list.ordered {
        if bullet == '.' { ')' } else { '.' }
    } else {
        settings.bullet_other
    };
    let mut use_different = parent.is_some()
        && state
            .bullet_last_used
            .as_deref()
            .is_some_and(|last| last == bullet.to_string());
    if !list.ordered {
        let first = list.children.first();
        let stack = &state.stack;
        let indexes = &state.index_stack;
        let from_end = |values: &[Construct], back: usize| {
            values.len().checked_sub(back).map(|index| values[index])
        };
        let index_from_end =
            |back: usize| indexes.len().checked_sub(back).map(|index| indexes[index]);
        // `- *` followed by an empty item would read as a thematic break
        if matches!(bullet, '*' | '-')
            && first.is_some_and(|item| {
                item.children().is_none_or(|children| children.is_empty())
            })
            && from_end(stack, 1) == Some(Construct::List)
            && from_end(stack, 2) == Some(Construct::ListItem)
            && from_end(stack, 3) == Some(Construct::List)
            && from_end(stack, 4) == Some(Construct::ListItem)
            && index_from_end(1) == Some(0)
            && index_from_end(2) == Some(0)
            && index_from_end(3) == Some(0)
        {
            use_different = true;
        }
        if settings.rule == bullet
            && first.is_some()
            && list.children.iter().any(|item| {
                matches!(item, Node::ListItem(item)
                    if matches!(item.children.first(), Some(Node::ThematicBreak(_))))
            })
        {
            use_different = true;
        }
    }
    if use_different {
        bullet = bullet_other;
    }
    state.bullet_current = Some(bullet.to_string());
    let result = state.container_flow(node);
    state.bullet_last_used = Some(bullet.to_string());
    state.bullet_current = bullet_current;
    state.exit();
    result
}
