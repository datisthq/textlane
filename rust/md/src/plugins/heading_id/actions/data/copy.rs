use textlane_core::{Allocator, ArenaVec, Data};

/// A copy of unist `data` in the arena, as deep as it nests.
pub(crate) fn copy_data<'a>(data: &Data<'a>, alloc: &'a Allocator) -> Data<'a> {
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || match data {
        Data::Null => Data::Null,
        Data::Boolean(value) => Data::Boolean(*value),
        Data::Number(value) => Data::Number(*value),
        Data::String(value) => Data::String(value),
        Data::Array(items) => {
            let mut copied = ArenaVec::with_capacity_in(items.len(), alloc);
            copied.extend(items.iter().map(|item| copy_data(item, alloc)));
            Data::Array(copied)
        }
        Data::Object(entries) => {
            let mut copied = ArenaVec::with_capacity_in(entries.len(), alloc);
            copied.extend(
                entries
                    .iter()
                    .map(|(key, value)| (*key, copy_data(value, alloc))),
            );
            Data::Object(copied)
        }
    })
}
