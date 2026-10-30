use std::sync::atomic::Ordering;

use crate::actions::memory::set::RETAINED_MEMORY;

/// The most memory, in bytes, each thread keeps after a document to reuse for the next.
pub fn read_retained_memory() -> usize {
    RETAINED_MEMORY.load(Ordering::Relaxed)
}
