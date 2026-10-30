use std::cell::RefCell;

use textlane_core::Allocator;

use crate::read_retained_memory;

// A document's trees die with it; freeing the arena hands its pages back to the system and
// the next document faults them in again, so one arena stays with the thread, emptied

thread_local! {
    static SPARE: RefCell<Option<Allocator>> = const { RefCell::new(None) };
}

/// Run `run` with an empty arena, reusing the thread's last one when it is free.
pub(crate) fn with_allocator<R>(run: impl FnOnce(&Allocator) -> R) -> R {
    let mut alloc = SPARE.with_borrow_mut(Option::take).unwrap_or_default();
    let result = run(&alloc);
    // `reset` keeps only the largest chunk, so the limit weighs what stays, not the peak
    alloc.reset();
    if alloc.allocated_bytes() <= read_retained_memory() {
        SPARE.with_borrow_mut(|spare| *spare = Some(alloc));
    }
    result
}

/// The size of the arena the thread keeps, if any.
#[cfg(test)]
pub(crate) fn read_spare_bytes() -> Option<usize> {
    SPARE.with_borrow(|spare| spare.as_ref().map(Allocator::allocated_bytes))
}
