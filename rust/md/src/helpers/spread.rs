use std::collections::{HashMap, HashSet};

use crate::models::directive::{DIRECTIVE_CONTAINER, DIRECTIVE_CONTAINER_CONTENT};
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
pub(crate) fn find_loose(
    events: &[Event],
    runs: &[(usize, TrailingRun)],
) -> HashSet<usize> {
    let mut loose = HashSet::new();
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
                && blank < read_trailing(runs, index).start
            {
                loose.insert(enter);
            }
            continue;
        }
        if event.name != Name::BLANK_LINE_ENDING {
            continue;
        }
        // A container directive is not a container to lists, so its blank lines are their
        // items' own
        let Some(&parent) = open.iter().rev().find(|enter| {
            let name = events[**enter].name;
            name != DIRECTIVE_CONTAINER && name != DIRECTIVE_CONTAINER_CONTENT
        }) else {
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

/// The run of line endings and blank lines right before an event, stepping over the exits
/// of lists inside.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TrailingRun {
    /// Where the run starts, block quote prefixes stepped over too.
    pub(crate) start: usize,
    /// The first line ending in the run, without and with block quote prefixes.
    pub(crate) line_ending: [Option<usize>; 2],
}

/// The trailing run before the exit of each list and list item, the only events that ask,
/// by the exit's index; found in one pass, as nested lists all end at the same place.
pub(crate) fn find_trailing(events: &[Event]) -> Vec<(usize, TrailingRun)> {
    let mut runs = vec![];
    let mut run = TrailingRun::default();
    for (index, event) in events.iter().enumerate() {
        let exit = event.kind == Kind::Exit && CONTAINERS.contains(&event.name);
        if exit {
            runs.push((index, run));
        }
        let trailing = TRAILING.contains(&event.name) || exit;
        let prefix = PREFIXES.contains(&event.name);
        let line_ending = event.kind == Kind::Enter
            && (event.name == Name::LINE_ENDING || event.name == Name::BLANK_LINE_ENDING);
        if !(trailing || prefix) {
            run.start = index + 1;
        }
        for (first, trailing) in run
            .line_ending
            .iter_mut()
            .zip([trailing, trailing || prefix])
        {
            *first = match (trailing, *first) {
                (false, _) => None,
                (true, Some(first)) => Some(first),
                (true, None) => line_ending.then_some(index),
            };
        }
    }
    runs
}

/// The trailing run before the event at `index`, when it is the exit of a list or item.
pub(crate) fn read_trailing(runs: &[(usize, TrailingRun)], index: usize) -> TrailingRun {
    runs.binary_search_by_key(&index, |(at, _)| *at)
        .map(|found| runs[found].1)
        .unwrap_or_default()
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
