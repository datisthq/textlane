use crate::models::position::{Point, Position, UnistPoint, UnistPosition};

/// The line starts of a source, to turn byte points into unist's UTF-16 lines and columns.
#[derive(Clone, Debug)]
pub struct Lines<'a> {
    source: &'a str,
    starts: Vec<usize>,
    ascii: bool,
}

impl<'a> Lines<'a> {
    pub fn new(source: &'a str) -> Self {
        let bytes = source.as_bytes();
        let mut starts = vec![0];
        let mut index = 0;
        while index < bytes.len() {
            match bytes[index] {
                b'\n' => starts.push(index + 1),
                b'\r' if bytes.get(index + 1) != Some(&b'\n') => starts.push(index + 1),
                _ => {}
            }
            index += 1;
        }
        Self {
            source,
            starts,
            ascii: source.is_ascii(),
        }
    }

    /// The unist point of a byte point.
    pub fn to_unist_point(&self, point: Point) -> UnistPoint {
        let offset = point.offset().min(self.source.len());
        let line = self.starts.partition_point(|start| *start <= offset);
        let start = self.starts[line - 1];
        UnistPoint {
            line,
            column: self.count_units(start, offset) + 1 + point.virtual_spaces(),
            offset: Some(self.count_units(0, offset)),
        }
    }

    /// The unist position of a position, which is unchanged when it is one already.
    pub fn to_unist_position(&self, position: &Position<'_>) -> UnistPosition {
        match position {
            Position::Span { start, end } => UnistPosition {
                start: self.to_unist_point(*start),
                end: self.to_unist_point(*end),
            },
            Position::Unist(position) => (*position).clone(),
        }
    }

    fn count_units(&self, from: usize, to: usize) -> usize {
        if self.ascii {
            return to - from;
        }
        self.source
            .get(from..to)
            .map_or(to - from, |slice| slice.encode_utf16().count())
    }
}

#[cfg(test)]
#[path = "lines_unit.rs"]
mod unit;
