use std::cell::RefCell;

use textlane_core::Allocator;

// A document's trees die with it; freeing the arena hands its pages back to the system and
// the next document faults them in again, so one arena stays with the thread, emptied
const MAX_KEPT: usize = 64 << 20;

thread_local! {
    static SPARE: RefCell<Option<Allocator>> = const { RefCell::new(None) };
}

/// Run `run` with an empty arena, reusing the thread's last one when it is free.
pub(crate) fn with_allocator<R>(run: impl FnOnce(&Allocator) -> R) -> R {
    let mut alloc = SPARE.with_borrow_mut(Option::take).unwrap_or_default();
    let result = run(&alloc);
    if alloc.allocated_bytes() <= MAX_KEPT {
        alloc.reset();
        SPARE.with_borrow_mut(|spare| *spare = Some(alloc));
    }
    result
}
