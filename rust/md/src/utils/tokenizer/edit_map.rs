use std::cell::RefCell;

use crate::models::event::{Event, Link};

/// Edits to an event list, collected and then applied in one pass, keeping links valid.
#[derive(Debug, Default)]
pub(crate) struct EditMap {
    edits: Vec<Edit>,
}

// One call's edit: those at the same index merge once sorted, which keeps their call order
#[derive(Debug)]
struct Edit {
    at: usize,
    remove: usize,
    before: bool,
    add: Vec<Event>,
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

    fn push(&mut self, at: usize, remove: usize, before: bool, add: Vec<Event>) {
        if remove != 0 || !add.is_empty() {
            self.edits.push(Edit {
                at,
                remove,
                before,
                add,
            });
        }
    }

    /// Apply the edits, rebuilding `events` into `spare` when it is large enough and keeping
    /// the larger of the two buffers there.
    pub(crate) fn consume(
        &mut self,
        events: &mut Vec<Event>,
        spare: &RefCell<Vec<Event>>,
    ) {
        if self.edits.is_empty() {
            return;
        }
        let mut edits = std::mem::take(&mut self.edits);
        edits.sort_by_key(|edit| edit.at);
        // Each index's edits, as `(at, remove, start, end)` ranges of `edits`
        let mut groups: Vec<(usize, usize, usize, usize)> = vec![];
        let mut jumps = vec![];
        let mut add_acc = 0;
        let mut remove_acc = 0;
        for (index, edit) in edits.iter().enumerate() {
            match groups.last_mut() {
                Some(group) if group.0 == edit.at => {
                    group.1 += edit.remove;
                    group.3 = index + 1;
                }
                _ => groups.push((edit.at, edit.remove, index, index + 1)),
            }
            remove_acc += edit.remove;
            add_acc += edit.add.len();
            if jumps
                .last()
                .is_some_and(|jump: &(usize, usize, usize)| jump.0 == edit.at)
            {
                jumps.pop();
            }
            jumps.push((edit.at, remove_acc, add_acc));
        }
        // One pass that shifts links and copies each kept event and each added one once
        let len = events.len() + add_acc - remove_acc;
        let mut old = std::mem::take(events);
        let mut spare = spare.borrow_mut();
        if spare.capacity() >= len {
            std::mem::swap(events, &mut spare);
        } else {
            events.reserve(len);
        }
        let mut links = Links::new(&jumps);
        let mut index = 0;
        for (at, remove, start, end) in groups {
            let kept = at.clamp(index, old.len());
            links.shift_to(&mut old, kept);
            events.extend_from_slice(&old[index..kept]);
            // Chunks added before come latest first: prepending in place would copy
            for edit in edits[start..end]
                .iter_mut()
                .rev()
                .filter(|edit| edit.before)
            {
                events.append(&mut edit.add);
            }
            for edit in edits[start..end].iter_mut().filter(|edit| !edit.before) {
                events.append(&mut edit.add);
            }
            index = (kept + remove).min(old.len());
        }
        let end = old.len();
        links.shift_to(&mut old, end);
        events.extend_from_slice(&old[index..]);
        if old.capacity() > spare.capacity() {
            old.clear();
            *spare = old;
        }
    }
}

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
