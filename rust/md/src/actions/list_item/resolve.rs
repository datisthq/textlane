use std::collections::HashMap;

use crate::helpers::skip::{skip_opt, skip_to};
use crate::models::event::{Event, Kind};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

// A list in progress: its marker, nesting depth, first item's enter and last item's exit
type List = (u8, usize, usize, usize);

/// Group adjacent list items with the same marker into lists.
pub(crate) fn resolve_list_item(tokenizer: &mut Tokenizer) {
    let mut lists_wip: Vec<List> = vec![];
    let mut lists: Vec<List> = vec![];
    let exits = find_item_exits(&tokenizer.events);
    let mut balance = 0;
    for index in 0..tokenizer.events.len() {
        let event = &tokenizer.events[index];
        if event.name != Name::LIST_ITEM {
            continue;
        }
        if event.kind == Kind::Exit {
            balance -= 1;
            continue;
        }
        let end = exits.get(&index).copied().unwrap_or(index);
        let marker = skip_to(&tokenizer.events, index, &[Name::LIST_ITEM_MARKER]);
        let marker = tokenizer.parse_state.bytes[tokenizer.events[marker].point.index];
        let current = (marker, balance, index, end);
        let mut matched = false;
        let mut list_index = lists_wip.len();
        while list_index > 0 {
            list_index -= 1;
            let previous = lists_wip[list_index];
            // Lists in progress nest deeper towards the end: a shallower one ends the search
            if previous.1 < current.1 {
                break;
            }
            // The previous item of this list ends right before this one, give or take
            // whitespace, line endings and block quote prefixes
            let before = skip_opt(
                &tokenizer.events,
                previous.3 + 1,
                &[
                    Name::SPACE_OR_TAB,
                    Name::LINE_ENDING,
                    Name::BLANK_LINE_ENDING,
                    Name::BLOCK_QUOTE_PREFIX,
                ],
            );
            if previous.0 == current.0 && previous.1 == current.1 && before == current.2 {
                lists_wip[list_index].3 = current.3;
                lists.append(&mut lists_wip.split_off(list_index + 1));
                matched = true;
                break;
            }
        }
        if !matched {
            let mut exit = None;
            let mut wip_index = lists_wip.len();
            while wip_index > 0 && current.2 > lists_wip[wip_index - 1].3 {
                wip_index -= 1;
                exit = Some(wip_index);
            }
            if let Some(exit) = exit {
                lists.append(&mut lists_wip.split_off(exit));
            }
            lists_wip.push(current);
        }
        balance += 1;
    }
    lists.append(&mut lists_wip);
    for (marker, _, start, end) in lists {
        let name = match marker {
            b'.' | b')' => Name::LIST_ORDERED,
            _ => Name::LIST_UNORDERED,
        };
        let mut list_start = tokenizer.events[start].clone();
        let mut list_end = tokenizer.events[end].clone();
        list_start.name = name;
        list_end.name = name;
        tokenizer.map.add(start, 0, vec![list_start]);
        tokenizer.map.add(end + 1, 0, vec![list_end]);
    }
    tokenizer
        .map
        .consume(&mut tokenizer.events, &tokenizer.parse_state.spare_events);
}

// The exit of each list item, by its enter, found in one pass: walking each item's events
// would cost the nesting depth once per item
fn find_item_exits(events: &[Event]) -> HashMap<usize, usize> {
    let mut exits = HashMap::new();
    let mut open = vec![];
    for (index, event) in events.iter().enumerate() {
        if event.name != Name::LIST_ITEM {
            continue;
        }
        match event.kind {
            Kind::Enter => open.push(index),
            Kind::Exit => {
                if let Some(enter) = open.pop() {
                    exits.insert(enter, index);
                }
            }
        }
    }
    exits
}
