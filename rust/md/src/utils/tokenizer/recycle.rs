use std::cell::RefCell;

use crate::models::event::Event;

// Document-sized event lists freed per document make the allocator return their pages and
// fault them in again for the next one; a few stay with the thread, small enough to keep
const KEEP: usize = 3;
const MIN_CAPACITY: usize = 1 << 13;
const MAX_CAPACITY: usize = 1 << 18;

thread_local! {
    static SPARE: RefCell<Vec<Vec<Event>>> = const { RefCell::new(Vec::new()) };
}

/// An empty event list, with room from an earlier document when one is spare.
pub(crate) fn take_events() -> Vec<Event> {
    SPARE.with_borrow_mut(Vec::pop).unwrap_or_default()
}

/// Keep `events`' room for a later document, when it is worth keeping.
pub(crate) fn give_events(mut events: Vec<Event>) {
    let capacity = events.capacity();
    if !(MIN_CAPACITY..=MAX_CAPACITY).contains(&capacity) {
        return;
    }
    events.clear();
    SPARE.with_borrow_mut(|spare| {
        if spare.len() < KEEP {
            spare.push(events);
        } else if let Some(smallest) = spare.iter_mut().min_by_key(|kept| kept.capacity())
            && smallest.capacity() < capacity
        {
            *smallest = events;
        }
    });
}
