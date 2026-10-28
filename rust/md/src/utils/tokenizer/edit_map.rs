use std::collections::HashMap;

use crate::models::event::Event;

/// Edits to an event list, collected and then applied in one pass, keeping links valid.
#[derive(Debug, Default)]
pub(crate) struct EditMap {
    map: Vec<Edit>,
    // Where each index's edit is in `map`, so adding stays constant time
    entries: HashMap<usize, usize>,
}

#[derive(Debug)]
struct Edit {
    at: usize,
    remove: usize,
    // Chunks added before, latest first once reversed: prepending in place would copy
    before: Vec<Vec<Event>>,
    after: Vec<Event>,
}

impl EditMap {
    pub(crate) fn new() -> EditMap {
        EditMap::default()
    }

    /// At `index`, remove `remove` events and add `add` after what is already added there.
    pub(crate) fn add(&mut self, index: usize, remove: usize, mut add: Vec<Event>) {
        if let Some(edit) = self.edit_mut(index, remove, add.is_empty()) {
            edit.remove += remove;
            edit.after.append(&mut add);
        }
    }

    /// At `index`, remove `remove` events and add `add` before what is already added there.
    pub(crate) fn add_before(&mut self, index: usize, remove: usize, add: Vec<Event>) {
        if let Some(edit) = self.edit_mut(index, remove, add.is_empty()) {
            edit.remove += remove;
            edit.before.push(add);
        }
    }

    fn edit_mut(&mut self, at: usize, remove: usize, empty: bool) -> Option<&mut Edit> {
        if remove == 0 && empty {
            return None;
        }
        let next = self.map.len();
        let index = *self.entries.entry(at).or_insert(next);
        if index == next {
            self.map.push(Edit {
                at,
                remove: 0,
                before: vec![],
                after: vec![],
            });
        }
        self.map.get_mut(index)
    }

    pub(crate) fn consume(&mut self, events: &mut Vec<Event>) {
        if self.map.is_empty() {
            return;
        }
        self.map.sort_unstable_by_key(|edit| edit.at);
        let map: Vec<(usize, usize, Vec<Event>)> = self
            .map
            .drain(..)
            .map(|mut edit| {
                let mut add = vec![];
                while let Some(mut chunk) = edit.before.pop() {
                    add.append(&mut chunk);
                }
                add.append(&mut edit.after);
                (edit.at, edit.remove, add)
            })
            .collect();
        self.entries.clear();
        let mut jumps = Vec::with_capacity(map.len());
        let mut add_acc = 0;
        let mut remove_acc = 0;
        for (at, remove, add) in &map {
            remove_acc += remove;
            add_acc += add.len();
            jumps.push((*at, remove_acc, add_acc));
        }
        shift_links(events, &jumps);
        // One pass that copies each kept event and each added one exactly once
        let old = std::mem::take(events);
        events.reserve(old.len() + add_acc - remove_acc);
        let mut old = old.into_iter().enumerate().peekable();
        for (at, remove, add) in map {
            while let Some((_, event)) = old.next_if(|(index, _)| *index < at) {
                events.push(event);
            }
            events.extend(add);
            for _ in 0..remove {
                old.next();
            }
        }
        events.extend(old.map(|(_, event)| event));
    }
}

fn shift_links(events: &mut [Event], jumps: &[(usize, usize, usize)]) {
    let mut jump_index = 0;
    let mut index = 0;
    let mut add = 0;
    let mut rm = 0;
    while index < events.len() {
        let rm_curr = rm;
        while jump_index < jumps.len() && jumps[jump_index].0 <= index {
            add = jumps[jump_index].2;
            rm = jumps[jump_index].1;
            jump_index += 1;
        }
        // Skip events that are removed
        if rm > rm_curr {
            index += rm - rm_curr;
            continue;
        }
        let next = events[index].link.as_ref().and_then(|link| link.next);
        if let Some(next) = next {
            if let Some(link) = events[next].link.as_mut() {
                link.previous = Some(index + add - rm);
            }
            while jump_index < jumps.len() && jumps[jump_index].0 <= next {
                add = jumps[jump_index].2;
                rm = jumps[jump_index].1;
                jump_index += 1;
            }
            if let Some(link) = events[index].link.as_mut() {
                link.next = Some(next + add - rm);
            }
            index = next;
            continue;
        }
        index += 1;
    }
}
