use crate::models::event::Event;

/// Edits to an event list, collected and then applied in one pass, keeping links valid.
#[derive(Debug, Default)]
pub(crate) struct EditMap {
    map: Vec<(usize, usize, Vec<Event>)>,
}

impl EditMap {
    pub(crate) fn new() -> EditMap {
        EditMap { map: vec![] }
    }

    /// At `index`, remove `remove` events and add `add` after what is already added there.
    pub(crate) fn add(&mut self, index: usize, remove: usize, add: Vec<Event>) {
        self.add_impl(index, remove, add, false);
    }

    // Used by resolvers that wrap tokens, such as attention, once they exist
    #[allow(dead_code)]
    /// At `index`, remove `remove` events and add `add` before what is already added there.
    pub(crate) fn add_before(&mut self, index: usize, remove: usize, add: Vec<Event>) {
        self.add_impl(index, remove, add, true);
    }

    fn add_impl(&mut self, at: usize, remove: usize, mut add: Vec<Event>, before: bool) {
        if remove == 0 && add.is_empty() {
            return;
        }
        for entry in &mut self.map {
            if entry.0 == at {
                entry.1 += remove;
                if before {
                    add.append(&mut entry.2);
                    entry.2 = add;
                } else {
                    entry.2.append(&mut add);
                }
                return;
            }
        }
        self.map.push((at, remove, add));
    }

    pub(crate) fn consume(&mut self, events: &mut Vec<Event>) {
        self.map.sort_unstable_by_key(|entry| entry.0);
        if self.map.is_empty() {
            return;
        }
        let mut jumps = Vec::with_capacity(self.map.len());
        let mut add_acc = 0;
        let mut remove_acc = 0;
        for (at, remove, add) in &self.map {
            remove_acc += remove;
            add_acc += add.len();
            jumps.push((*at, remove_acc, add_acc));
        }
        shift_links(events, &jumps);
        let len_before = events.len();
        let mut index = self.map.len();
        let mut vecs = Vec::with_capacity(index * 2 + 1);
        while index > 0 {
            index -= 1;
            vecs.push(events.split_off(self.map[index].0 + self.map[index].1));
            vecs.push(self.map[index].2.split_off(0));
            events.truncate(self.map[index].0);
        }
        vecs.push(events.split_off(0));
        events.reserve(len_before + add_acc - remove_acc);
        while let Some(mut slice) = vecs.pop() {
            events.append(&mut slice);
        }
        self.map.clear();
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
