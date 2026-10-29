use std::cmp::Ordering;

use textlane_core::{File, Message, Point, Position, UnistPoint};

use crate::helpers::place::to_place;
use crate::models::report::Report;

/// Add a check's reports to the file as messages in place order (start offset, then rule),
/// as upstream's tests sort them. Byte spans are placed in one sweep over the source, since
/// counting UTF-16 units from the start once per message is quadratic.
pub(crate) fn place_messages(file: &mut File, reports: Vec<Report<'_>>) {
    let mut points: Vec<(Point, usize)> = Vec::new();
    for (index, report) in reports.iter().enumerate() {
        if let Some(Position::Span { start, end }) = report.position {
            points.push((start, index * 2));
            points.push((end, index * 2 + 1));
        }
    }
    points.sort_unstable_by_key(|(point, _)| point.offset());
    let mut placed = vec![None; reports.len() * 2];
    if let Some(source) = file.value.as_deref() {
        let mut sweep = Sweep::new(source);
        for (point, slot) in points {
            placed[slot] = Some(sweep.advance(point));
        }
    }
    let mut messages: Vec<Message> = reports
        .into_iter()
        .enumerate()
        .map(|(index, report)| {
            let place = match &report.position {
                Some(Position::Span { .. }) => placed[index * 2]
                    .zip(placed[index * 2 + 1])
                    .map(|(start, end)| textlane_core::UnistPosition { start, end }),
                Some(position) => to_place(file, position),
                None => None,
            };
            Message {
                reason: report.reason,
                source: Some(report.source.to_string()),
                rule: Some(report.rule),
                place,
            }
        })
        .collect();
    messages.sort_by(compare_messages);
    file.messages.extend(messages);
}

// Lines and UTF-16 units counted forward, as `Lines::to_unist_point` counts them
struct Sweep<'s> {
    source: &'s str,
    byte: usize,
    line: usize,
    unit: usize,
    line_unit: usize,
}

impl<'s> Sweep<'s> {
    fn new(source: &'s str) -> Self {
        Self {
            source,
            byte: 0,
            line: 1,
            unit: 0,
            line_unit: 0,
        }
    }

    fn advance(&mut self, point: Point) -> UnistPoint {
        let offset = point.offset().min(self.source.len());
        let bytes = self.source.as_bytes();
        while self.byte < offset {
            let Some(character) = self.source[self.byte..].chars().next() else {
                break;
            };
            self.byte += character.len_utf8();
            self.unit += character.len_utf16();
            let breaks = character == '\n'
                || (character == '\r' && bytes.get(self.byte) != Some(&b'\n'));
            if breaks {
                self.line += 1;
                self.line_unit = self.unit;
            }
        }
        let extra = usize::from(point.virtual_spaces() > 0) + (point.offset() - offset);
        UnistPoint {
            line: self.line,
            column: self.unit - self.line_unit + 1 + extra,
            offset: Some(self.unit + extra),
        }
    }
}

fn compare_messages(left: &Message, right: &Message) -> Ordering {
    let start =
        |message: &Message| message.place.as_ref().and_then(|place| place.start.offset);
    start(left)
        .cmp(&start(right))
        .then_with(|| left.rule.cmp(&right.rule))
}
