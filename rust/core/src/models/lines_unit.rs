use super::Lines;
use crate::models::position::{Point, UnistPoint};

#[test]
fn counts_lines_after_each_line_ending() {
    let lines = Lines::new("a\nb\r\nc\rd");
    let points =
        [0, 2, 5, 7].map(|offset| lines.to_unist_point(Point::new(offset, 0)).line);
    assert_eq!(points, [1, 2, 3, 4]);
}

#[test]
fn counts_columns_and_offsets_in_utf16_units() {
    let lines = Lines::new("a😀b\n😀c");
    let point = lines.to_unist_point(Point::new("a😀b\n😀".len(), 0));
    let expected = UnistPoint {
        line: 2,
        column: 3,
        offset: Some(7),
    };
    assert_eq!(point, expected);
}

#[test]
fn adds_virtual_spaces_to_the_column() {
    let lines = Lines::new("\ta");
    let point = lines.to_unist_point(Point::new(0, 2));
    assert_eq!((point.column, point.offset), (3, Some(0)));
}
