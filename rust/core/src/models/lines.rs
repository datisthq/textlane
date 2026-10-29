use crate::models::position::{Point, Position, UnistPoint, UnistPosition};

/// The line starts of a source, to turn byte points into unist's UTF-16 lines and columns.
#[derive(Clone, Debug)]
pub struct Lines<'a> {
    source: &'a str,
    starts: Vec<usize>,
    ascii: bool,
    // UTF-16 units before every `BLOCK`th byte, so a point never counts from the start
    checkpoints: Vec<usize>,
}

const BLOCK: usize = 256;

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
        let ascii = source.is_ascii();
        let checkpoints = match ascii {
            true => vec![],
            false => {
                let mut total = 0;
                let mut checkpoints = Vec::with_capacity(bytes.len() / BLOCK + 1);
                for block in bytes.chunks(BLOCK) {
                    checkpoints.push(total);
                    total += count_byte_units(block);
                }
                checkpoints
            }
        };
        Self {
            source,
            starts,
            ascii,
            checkpoints,
        }
    }

    /// The unist point of a byte point.
    pub fn to_unist_point(&self, point: Point) -> UnistPoint {
        let offset = point.offset().min(self.source.len());
        let line = self.starts.partition_point(|start| *start <= offset);
        let start = self.starts[line - 1];
        // Inside a tab, however far into its columns, micromark counts the tab as passed by
        // one column; past the end, as parse5 puts a token ended by EOF, counts on the last line
        let extra = usize::from(point.virtual_spaces() > 0) + (point.offset() - offset);
        UnistPoint {
            line,
            column: self.count_units_to(offset) - self.count_units_to(start) + 1 + extra,
            offset: Some(self.count_units_to(offset) + extra),
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

    // UTF-16 units before byte `to`, from the checkpoint before it
    fn count_units_to(&self, to: usize) -> usize {
        if self.ascii {
            return to;
        }
        let block = to / BLOCK;
        let before = self.checkpoints.get(block).copied().unwrap_or_default();
        let bytes = self.source.as_bytes();
        before + count_byte_units(bytes.get(block * BLOCK..to).unwrap_or_default())
    }
}

// Each UTF-8 lead byte starts one UTF-16 unit, or two for a four-byte character
fn count_byte_units(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .map(|byte| match byte {
            0x80..=0xBF => 0,
            0xF0..=0xFF => 2,
            _ => 1,
        })
        .sum()
}

#[cfg(test)]
#[path = "lines_unit.rs"]
mod unit;
