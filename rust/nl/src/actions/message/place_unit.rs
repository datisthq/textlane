use textlane_core::{File, Lines, Point, Position};

use super::place_messages;
use crate::models::report::Report;

#[test]
fn places_messages_as_lines_does_whatever_their_order() {
    let source = "a😀b\r\nc\rd\n\te";
    let spans = [(12, 14), (0, 1), (5, 8), (9, 9)];
    let positions: Vec<_> = spans
        .iter()
        .map(|(start, end)| Position::span(Point::new(*start, 0), Point::new(*end, 0)))
        .collect();
    let mut file = File {
        value: Some(source.to_string()),
        ..File::new()
    };
    let reports = (positions.iter())
        .map(|position| Report {
            reason: String::new(),
            rule: String::new(),
            source: "test",
            position: Some(*position),
        })
        .collect();
    place_messages(&mut file, reports);
    let lines = Lines::new(source);
    let expected: Vec<_> = (positions.iter())
        .map(|position| Some(lines.to_unist_position(position)))
        .collect();
    let actual: Vec<_> = file
        .messages
        .into_iter()
        .map(|message| message.place)
        .collect();
    assert_eq!(actual, expected);
}
