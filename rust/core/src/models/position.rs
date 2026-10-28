/// A place in the source: a byte offset, plus the virtual spaces into a tab it sits at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point(u32);

// The top two bits hold the virtual spaces (a tab spans at most three of them)
const VIRTUAL_SHIFT: u32 = 30;
const OFFSET_MASK: u32 = (1 << VIRTUAL_SHIFT) - 1;

impl Point {
    /// The largest byte offset a point can hold.
    pub const MAX_OFFSET: usize = OFFSET_MASK as usize;

    pub fn new(offset: usize, virtual_spaces: usize) -> Self {
        let offset = offset.min(Self::MAX_OFFSET) as u32;
        let virtual_spaces = virtual_spaces.min(3) as u32;
        Self(offset | (virtual_spaces << VIRTUAL_SHIFT))
    }

    pub fn offset(self) -> usize {
        (self.0 & OFFSET_MASK) as usize
    }

    pub fn virtual_spaces(self) -> usize {
        (self.0 >> VIRTUAL_SHIFT) as usize
    }
}

/// Where a node sits: a span of the current source, or a unist position that does not point
/// into it, such as one a foreign plugin set or one read from a serialized tree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Position<'a> {
    Span { start: Point, end: Point },
    Unist(&'a UnistPosition),
}

/// A position as unist writes it: lines and columns from 1, offsets in UTF-16 code units.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnistPosition {
    pub start: UnistPoint,
    pub end: UnistPoint,
}

/// A point as unist writes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnistPoint {
    pub line: usize,
    pub column: usize,
    pub offset: Option<usize>,
}

impl<'a> Position<'a> {
    pub fn span(start: Point, end: Point) -> Self {
        Self::Span { start, end }
    }
}
