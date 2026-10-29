use std::ops::Range;

use crate::models::event::{Event, Kind, Link, Point};
use crate::models::token::Name;
use crate::utils::tokenizer::recycle::give_events;

/// Edits to an event list, collected and then applied in one pass, keeping links valid.
#[derive(Debug, Default)]
pub(crate) struct EditMap {
    edits: Vec<Edit>,
    // Events that pooled edits add, so a split child list needs no vector per piece
    pool: Vec<Event>,
}

// One call's edit: those at the same index merge once sorted, which keeps their call order
#[derive(Debug)]
struct Edit {
    at: usize,
    remove: usize,
    before: bool,
    add: Vec<Event>,
    pooled: Range<usize>,
}

impl EditMap {
    pub(crate) fn new() -> EditMap {
        EditMap::default()
    }

    /// At `index`, remove `remove` events and add `add` after what is already added there.
    pub(crate) fn add(&mut self, index: usize, remove: usize, add: Vec<Event>) {
        self.push(index, remove, false, add);
    }

    /// At `index`, remove `remove` events and add `add` before what is already added there.
    pub(crate) fn add_before(&mut self, index: usize, remove: usize, add: Vec<Event>) {
        self.push(index, remove, true, add);
    }

    /// Move `events` into the pool and return where they start there.
    pub(crate) fn pool(&mut self, events: &mut Vec<Event>) -> usize {
        let start = self.pool.len();
        if start == 0 {
            std::mem::swap(&mut self.pool, events);
        } else {
            self.pool.append(events);
        }
        start
    }

    /// At `index`, remove `remove` events and add the pooled events in `range`.
    pub(crate) fn add_pooled(
        &mut self,
        index: usize,
        remove: usize,
        range: Range<usize>,
    ) {
        if remove != 0 || !range.is_empty() {
            self.edits.push(Edit {
                at: index,
                remove,
                before: false,
                add: vec![],
                pooled: range,
            });
        }
    }

    fn push(&mut self, at: usize, remove: usize, before: bool, add: Vec<Event>) {
        if remove != 0 || !add.is_empty() {
            self.edits.push(Edit {
                at,
                remove,
                before,
                add,
                pooled: 0..0,
            });
        }
    }

    /// Apply the edits in place, keeping links valid.
    pub(crate) fn consume(&mut self, events: &mut Vec<Event>) {
        if self.edits.is_empty() {
            return;
        }
        let mut edits = std::mem::take(&mut self.edits);
        edits.sort_by_key(|edit| edit.at);
        // Each index's edits, as `(at, remove, added, start, end)`, a range of `edits`
        let mut groups: Vec<(usize, usize, usize, usize, usize)> = vec![];
        let mut jumps = vec![];
        let mut add_acc = 0;
        let mut remove_acc = 0;
        for (index, edit) in edits.iter().enumerate() {
            let added = edit.add.len() + edit.pooled.len();
            match groups.last_mut() {
                Some(group) if group.0 == edit.at => {
                    group.1 += edit.remove;
                    group.2 += added;
                    group.4 = index + 1;
                }
                _ => groups.push((edit.at, edit.remove, added, index, index + 1)),
            }
            remove_acc += edit.remove;
            add_acc += added;
            if jumps
                .last()
                .is_some_and(|jump: &(usize, usize, usize)| jump.0 == edit.at)
            {
                jumps.pop();
            }
            jumps.push((edit.at, remove_acc, add_acc));
        }
        let old_len = events.len();
        Links::new(&jumps).shift_to(events, old_len);
        // Kept runs as `(from, to, destination)`, and each group's added events' destination
        let mut runs = Vec::with_capacity(groups.len() + 1);
        let mut adds = Vec::with_capacity(groups.len());
        let mut index = 0;
        let mut written = 0;
        for &(at, remove, added, _, _) in &groups {
            let kept = at.clamp(index, old_len);
            runs.push((index, kept, written));
            written += kept - index;
            adds.push(written);
            written += added;
            index = (kept + remove).min(old_len);
        }
        runs.push((index, old_len, written));
        let len = written + old_len - index;
        if len > old_len {
            events.resize(len, FILLER);
        }
        // A run moving left lands on runs before it once they moved, one moving right on
        // runs after it: each direction goes in its own order or overwrites unmoved events
        for &(from, to, destination) in &runs {
            if destination < from {
                events.copy_within(from..to, destination);
            }
        }
        for &(from, to, destination) in runs.iter().rev() {
            if destination > from {
                events.copy_within(from..to, destination);
            }
        }
        for (group, &(_, _, _, start, end)) in groups.iter().enumerate() {
            let mut at = adds[group];
            // Chunks added before come latest first
            for edit in edits[start..end].iter().rev().filter(|edit| edit.before) {
                events[at..at + edit.add.len()].copy_from_slice(&edit.add);
                at += edit.add.len();
            }
            for edit in edits[start..end].iter().filter(|edit| !edit.before) {
                events[at..at + edit.add.len()].copy_from_slice(&edit.add);
                at += edit.add.len();
                let pooled = &self.pool[edit.pooled.clone()];
                events[at..at + pooled.len()].copy_from_slice(pooled);
                at += pooled.len();
            }
        }
        events.truncate(len);
        give_events(std::mem::take(&mut self.pool));
    }
}

const FILLER: Event = Event {
    kind: Kind::Enter,
    name: Name::DATA,
    point: Point {
        line: 0,
        column: 0,
        index: 0,
        vs: 0,
    },
    link: None,
};

/// markdown-rs's link shift, walked lazily: it visits only the events a chain leads to and
/// skips removed ones, so links it never reaches stay as they were.
struct Links<'j> {
    jumps: &'j [(usize, usize, usize)],
    jump_index: usize,
    index: usize,
    add: usize,
    rm: usize,
}

impl<'j> Links<'j> {
    fn new(jumps: &'j [(usize, usize, usize)]) -> Self {
        Links {
            jumps,
            jump_index: 0,
            index: 0,
            add: 0,
            rm: 0,
        }
    }

    /// Walk every step before `until`; a step only writes its own event and a later one.
    fn shift_to(&mut self, events: &mut [Event], until: usize) {
        while self.index < until && self.index < events.len() {
            let index = self.index;
            let rm_curr = self.rm;
            self.take_jumps(index);
            // Skip events that are removed
            if self.rm > rm_curr {
                self.index += self.rm - rm_curr;
                continue;
            }
            let next = events[index].link.as_ref().and_then(Link::next);
            if let Some(next) = next {
                if let Some(link) = events[next].link.as_mut() {
                    link.previous = Link::at(index + self.add - self.rm);
                }
                self.take_jumps(next);
                if let Some(link) = events[index].link.as_mut() {
                    link.next = Link::at(next + self.add - self.rm);
                }
                self.index = next;
                continue;
            }
            self.index += 1;
        }
    }

    fn take_jumps(&mut self, index: usize) {
        while let Some(&(at, rm, add)) = self.jumps.get(self.jump_index)
            && at <= index
        {
            self.add = add;
            self.rm = rm;
            self.jump_index += 1;
        }
    }
}
