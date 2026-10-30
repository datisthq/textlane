use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) static RETAINED_MEMORY: AtomicUsize = AtomicUsize::new(256 << 20);

/// Cap the memory, in bytes, each thread keeps after a document to reuse for the next;
/// `0` keeps none.
pub fn set_retained_memory(bytes: usize) {
    RETAINED_MEMORY.store(bytes, Ordering::Relaxed);
}

#[cfg(test)]
#[path = "set_unit.rs"]
mod unit;
