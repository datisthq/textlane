use crate::models::event::{Event, Kind, Point};

const TAB_SIZE: usize = 4;

/// The bytes a token spans, with the virtual spaces of tabs split at either edge.
#[derive(Debug)]
pub(crate) struct Slice<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) before: usize,
    pub(crate) after: usize,
}

impl<'a> Slice<'a> {
    pub(crate) fn from_points(bytes: &'a [u8], start: &Point, end: &Point) -> Slice<'a> {
        let mut before = start.vs;
        let mut after = end.vs;
        let mut start_index = start.index;
        let mut end_index = end.index;
        // The rest of a split tab runs to the next tab stop, which its column tells
        if before > 0 {
            before = TAB_SIZE - (start.column - 1) % TAB_SIZE;
            start_index += 1;
        }
        if after > 0 {
            after -= 1;
            end_index += 1;
        }
        Slice {
            bytes: bytes.get(start_index..end_index).unwrap_or_default(),
            before,
            after,
        }
    }

    /// The length in columns, virtual spaces included.
    pub(crate) fn len(&self) -> usize {
        self.bytes.len() + self.before + self.after
    }
}

/// The start and end point of the token whose exit is at `index`.
pub(crate) fn span_of_exit(events: &[Event], index: usize) -> (&Point, &Point) {
    let end = &events[index].point;
    let name = events[index].name;
    let mut start = index;
    while start > 0 && !(events[start].kind == Kind::Enter && events[start].name == name)
    {
        start -= 1;
    }
    (&events[start].point, end)
}
