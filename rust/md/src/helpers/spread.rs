use std::collections::{HashMap, HashSet};

use crate::models::event::{Event, Kind};
use crate::models::token::Name;

// What may sit between a list's content and its end without being content
const TRAILING: [Name; 3] = [
    Name::LINE_ENDING,
    Name::BLANK_LINE_ENDING,
    Name::SPACE_OR_TAB,
];
// Block quote prefixes count as content of a list, but not of its items
const PREFIXES: [Name; 2] = [Name::BLOCK_QUOTE_PREFIX, Name::BLOCK_QUOTE_MARKER];
// Lists inside close where their content does, before any trailing lines; a block quote
// inside keeps its trailing lines, so its exit stops the walk
const CONTAINERS: [Name; 3] = [Name::LIST_ITEM, Name::LIST_ORDERED, Name::LIST_UNORDERED];

/// The enters of every loose list and list item: one with a blank line between its
/// children, found in one pass over the events however deep lists nest.
///
/// As mdast-util-from-markdown's `prepareList`, blank lines that only trail an item's
/// content do not make it loose.
pub(crate) fn find_loose(events: &[Event]) -> HashSet<usize> {
    let mut loose = HashSet::new();
    let trailing_starts = find_trailing_starts(events, true);
    let mut open: Vec<usize> = vec![];
    // The first blank line inside each open item, by the item's enter
    let mut first_blank: HashMap<usize, usize> = HashMap::new();
    for (index, event) in events.iter().enumerate() {
        if event.kind == Kind::Enter {
            open.push(index);
            continue;
        }
        let enter = open.pop();
        if event.name == Name::LIST_ITEM {
            if let Some(enter) = enter
                && let Some(blank) = first_blank.remove(&enter)
                && blank < trailing_starts[index]
            {
                loose.insert(enter);
            }
            continue;
        }
        if event.name != Name::BLANK_LINE_ENDING {
            continue;
        }
        let Some(&parent) = open.last() else {
            continue;
        };
        let parent_name = events[parent].name;
        if parent_name == Name::LIST_ITEM {
            if !after_item_prefix(events, index) && !after_empty_container(events, index)
            {
                first_blank.entry(parent).or_insert(index);
            }
        } else if (parent_name == Name::LIST_ORDERED
            || parent_name == Name::LIST_UNORDERED)
            && !after_empty_container(events, index)
        {
            loose.insert(parent);
        }
    }
    loose
}

/// For each event, where the run of line endings and blank lines right before it starts,
/// stepping over the exits of lists inside; `prefixes` steps over block quote prefixes
/// too. Found in one pass, as nested lists all end at the same place.
pub(crate) fn find_trailing_starts(events: &[Event], prefixes: bool) -> Vec<usize> {
    let mut starts = Vec::with_capacity(events.len() + 1);
    starts.push(0);
    for (index, event) in events.iter().enumerate() {
        let trailing = TRAILING.contains(&event.name)
            || (prefixes && PREFIXES.contains(&event.name))
            || (event.kind == Kind::Exit && CONTAINERS.contains(&event.name));
        starts.push(if trailing { starts[index] } else { index + 1 });
    }
    starts
}

// A blank line right after an item's marker is its empty first line, not a gap
fn after_item_prefix(events: &[Event], exit: usize) -> bool {
    let Some(mut before) = exit.checked_sub(2) else {
        return false;
    };
    if events[before].name == Name::SPACE_OR_TAB {
        let Some(earlier) = before.checked_sub(2) else {
            return false;
        };
        before = earlier;
    }
    events[before].name == Name::LIST_ITEM_PREFIX
}

// A blank line closing empty containers opened on its own line: micromark puts it inside
// the innermost, so it separates nothing in the items around
fn after_empty_container(events: &[Event], exit: usize) -> bool {
    let mut before = exit.saturating_sub(1);
    let mut innermost = None;
    while let Some(earlier) = before.checked_sub(1) {
        before = earlier;
        let event = &events[before];
        let container =
            CONTAINERS.contains(&event.name) || event.name == Name::BLOCK_QUOTE;
        if !(event.kind == Kind::Exit && container) {
            break;
        }
        innermost = Some(event.name);
    }
    if events[before].name == Name::SPACE_OR_TAB {
        let Some(earlier) = before.checked_sub(2) else {
            return false;
        };
        before = earlier;
    }
    // The prefix must open the innermost closed container, not continue an outer one
    matches!(
        (events[before].name, innermost),
        (Name::LIST_ITEM_PREFIX, Some(Name::LIST_ITEM))
            | (Name::BLOCK_QUOTE_PREFIX, Some(Name::BLOCK_QUOTE))
    )
}

/// For each event, the first line ending in the run of trailing lines right before it,
/// as `find_trailing_starts` delimits that run.
pub(crate) fn find_trailing_line_endings(
    events: &[Event],
    prefixes: bool,
) -> Vec<Option<usize>> {
    let mut firsts = Vec::with_capacity(events.len() + 1);
    firsts.push(None);
    for (index, event) in events.iter().enumerate() {
        let trailing = TRAILING.contains(&event.name)
            || (prefixes && PREFIXES.contains(&event.name))
            || (event.kind == Kind::Exit && CONTAINERS.contains(&event.name));
        let line_ending = event.kind == Kind::Enter
            && (event.name == Name::LINE_ENDING || event.name == Name::BLANK_LINE_ENDING);
        firsts.push(match (trailing, firsts[index]) {
            (false, _) => None,
            (true, Some(first)) => Some(first),
            (true, None) => line_ending.then_some(index),
        });
    }
    firsts
}
