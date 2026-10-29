use crate::Allocator;
use crate::models::nlcst::Node;
use crate::models::position::{Point, Position, UnistPoint, UnistPosition};

/// A source indexed as vfile-location reads it: offsets in UTF-16 code units, turned into
/// lines, columns and the byte points of core's positions, each in constant or log time.
#[derive(Clone, Debug)]
pub struct Location<'a> {
    source: &'a str,
    starts: Vec<usize>,
    // Empty when the source is ASCII, where bytes and units coincide
    units: Vec<u32>,
    bytes: Vec<u32>,
}

// Marks the second unit of a surrogate pair, which no byte starts
const INSIDE: u32 = u32::MAX;

impl<'a> Location<'a> {
    pub fn new(source: &'a str) -> Self {
        let bytes = source.as_bytes();
        let mut starts = vec![0];
        for (index, byte) in bytes.iter().enumerate() {
            match byte {
                b'\n' => starts.push(index + 1),
                b'\r' if bytes.get(index + 1) != Some(&b'\n') => starts.push(index + 1),
                _ => {}
            }
        }
        let mut location = Self {
            source,
            starts,
            units: Vec::new(),
            bytes: Vec::new(),
        };
        if !source.is_ascii() {
            let mut units = Vec::with_capacity(source.len() + 1);
            let mut offsets = Vec::with_capacity(source.len() + 1);
            let mut unit = 0;
            for (index, character) in source.char_indices() {
                units.resize(index + character.len_utf8(), unit);
                offsets.push(index as u32);
                if character.len_utf16() == 2 {
                    offsets.push(INSIDE);
                }
                unit += character.len_utf16() as u32;
            }
            units.push(unit);
            offsets.push(source.len() as u32);
            location.units = units;
            location.bytes = offsets;
        }
        location
    }

    pub fn source(&self) -> &'a str {
        self.source
    }

    /// The source's length in UTF-16 code units.
    pub fn len(&self) -> usize {
        self.to_unit(self.source.len())
    }

    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }

    /// The UTF-16 offset of a byte offset, inside a character counting as its start.
    pub fn to_unit(&self, byte: usize) -> usize {
        let byte = byte.min(self.source.len());
        match self.units.get(byte) {
            Some(unit) => *unit as usize,
            None => byte,
        }
    }

    /// The byte offset of a UTF-16 offset, unless it is past the end or inside a pair.
    pub fn to_byte(&self, unit: usize) -> Option<usize> {
        if self.units.is_empty() {
            return (unit <= self.source.len()).then_some(unit);
        }
        match self.bytes.get(unit) {
            Some(&INSIDE) | None => None,
            Some(byte) => Some(*byte as usize),
        }
    }

    /// The unist point of a byte point, as `Lines` gives it, with its UTF-16 offset.
    pub fn to_unist_point(&self, point: Point) -> UnistPoint {
        let offset = point.offset().min(self.source.len());
        let line = self.line(offset);
        let extra = usize::from(point.virtual_spaces() > 0) + (point.offset() - offset);
        let unit = self.to_unit(offset);
        UnistPoint {
            line,
            column: unit - self.to_unit(self.starts[line - 1]) + 1 + extra,
            offset: Some(unit + extra),
        }
    }

    /// vfile-location's `toPoint`: the point at a UTF-16 offset, unless past the end.
    pub fn to_point(&self, unit: usize) -> Option<UnistPoint> {
        if unit > self.len() {
            return None;
        }
        let byte = match self.to_byte(unit) {
            Some(byte) => byte,
            None => self.to_byte(unit.checked_sub(1)?)?,
        };
        let line = self.line(byte);
        Some(UnistPoint {
            line,
            column: unit - self.to_unit(self.starts[line - 1]) + 1,
            offset: Some(unit),
        })
    }

    /// vfile-location's `toOffset`: the UTF-16 offset of a line and column, unless the
    /// column is past its line.
    pub fn to_offset(&self, line: usize, column: usize) -> Option<usize> {
        let start = self.to_unit(*self.starts.get(line.checked_sub(1)?)?);
        let next = match self.starts.get(line) {
            Some(next) => self.to_unit(*next),
            None => self.len() + 1,
        };
        let offset = (start + column).checked_sub(1)?;
        (offset < next).then_some(offset)
    }

    /// The position of a span of UTF-16 offsets: byte points when both ends start a
    /// character, unist points otherwise, none when either is past the end.
    pub fn to_position<'b>(
        &self,
        start: usize,
        end: usize,
        alloc: &'b Allocator,
    ) -> Option<Position<'b>> {
        match (self.to_byte(start), self.to_byte(end)) {
            (Some(start), Some(end)) => {
                Some(Position::span(Point::new(start, 0), Point::new(end, 0)))
            }
            _ => {
                let start = self.to_point(start)?;
                let end = self.to_point(end)?;
                Some(Position::Unist(alloc.alloc(UnistPosition { start, end })))
            }
        }
    }

    /// Place nlcst nodes one after another from a UTF-16 offset, as mdast-util-to-nlcst's
    /// and hast-util-to-nlcst's `patch` do: each spans its text's length, and a node
    /// without a start or past the end has no position. The offset after them.
    pub fn place_nlcst<'b>(
        &self,
        nodes: &mut [Node<'b>],
        start: Option<usize>,
        alloc: &'b Allocator,
    ) -> Option<usize> {
        let mut from = start;
        for node in nodes {
            let to = match node {
                Node::Root(parent)
                | Node::Paragraph(parent)
                | Node::Sentence(parent)
                | Node::Word(parent) => {
                    // Bounded: a bridge places what `tokenize` made, words of literals
                    let to = self.place_nlcst(&mut parent.children, from, alloc);
                    parent.position = self.place(from, to, alloc);
                    to
                }
                Node::Text(literal)
                | Node::Symbol(literal)
                | Node::Punctuation(literal)
                | Node::WhiteSpace(literal)
                | Node::Source(literal) => {
                    let to = from.map(|from| from + count_units(literal.value));
                    literal.position = self.place(from, to, alloc);
                    to
                }
                Node::Unknown(unknown) => {
                    unknown.position = None;
                    from
                }
            };
            from = to;
        }
        from
    }

    /// The position from one's start to another's end, as unist-util-position reads them.
    pub fn join<'b>(
        &self,
        start: &Position<'b>,
        end: &Position<'b>,
        alloc: &'b Allocator,
    ) -> Position<'b> {
        match (start, end) {
            (Position::Span { start, .. }, Position::Span { end, .. }) => {
                Position::span(*start, *end)
            }
            _ => Position::Unist(alloc.alloc(UnistPosition {
                start: self.to_unist_position(start).start,
                end: self.to_unist_position(end).end,
            })),
        }
    }

    fn to_unist_position(&self, position: &Position<'_>) -> UnistPosition {
        match position {
            Position::Span { start, end } => UnistPosition {
                start: self.to_unist_point(*start),
                end: self.to_unist_point(*end),
            },
            Position::Unist(position) => (*position).clone(),
        }
    }

    fn place<'b>(
        &self,
        from: Option<usize>,
        to: Option<usize>,
        alloc: &'b Allocator,
    ) -> Option<Position<'b>> {
        self.to_position(from?, to?, alloc)
    }

    // The line, from 1, a byte offset is on
    fn line(&self, byte: usize) -> usize {
        self.starts.partition_point(|start| *start <= byte)
    }
}

// The length of a string in UTF-16 code units, as JavaScript counts it
fn count_units(value: &str) -> usize {
    if value.is_ascii() {
        value.len()
    } else {
        value.chars().map(char::len_utf16).sum()
    }
}

#[cfg(test)]
#[path = "location_unit.rs"]
mod unit;
