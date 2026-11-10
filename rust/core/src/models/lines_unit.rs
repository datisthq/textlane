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
fn counts_a_point_inside_a_tab_as_past_its_first_column() {
    let lines = Lines::new("\ta");
    let point = lines.to_unist_point(Point::new(0, 2));
    assert_eq!((point.column, point.offset), (2, Some(1)));
}

#[test]
fn counts_utf16_offsets_across_checkpoints() {
    let source = "aé😀\nb".repeat(300);
    let lines = Lines::new(&source);
    for (index, _) in source.char_indices() {
        let point = lines.to_unist_point(Point::new(index, 0));
        let before = &source[..index];
        let line_start = before.rfind('\n').map_or(0, |at| at + 1);
        assert_eq!(point.offset, Some(before.encode_utf16().count()));
        assert_eq!(
            point.column,
            source[line_start..index].encode_utf16().count() + 1
        );
    }
}

#[test]
fn places_the_end_of_a_source_filling_whole_checkpoint_blocks() {
    // 256 bytes each, of two-, three- and four-byte characters
    let blocks = [
        "a\n".to_string() + &"é".repeat(127),
        "a\nbc".to_string() + &"’".repeat(84),
        "a\nbc".to_string() + &"😀".repeat(63),
    ];
    for block in blocks {
        for count in [1, 2] {
            let source = block.repeat(count);
            let line_start = source.rfind('\n').map_or(0, |at| at + 1);
            let point = Lines::new(&source).to_unist_point(Point::new(source.len(), 0));
            let expected = UnistPoint {
                line: source.matches('\n').count() + 1,
                column: source[line_start..].encode_utf16().count() + 1,
                offset: Some(source.encode_utf16().count()),
            };
            assert_eq!(point, expected, "{count} blocks of {block}");
        }
    }
}

#[test]
fn places_many_points_on_one_long_line_in_linear_time() {
    let source = "é ".repeat(500_000);
    let lines = Lines::new(&source);
    let started = std::time::Instant::now();
    for index in (0..source.len()).step_by(3) {
        lines.to_unist_point(Point::new(index, 0));
    }
    assert!(
        started.elapsed().as_secs() < if cfg!(debug_assertions) { 20 } else { 5 },
        "took {:?}",
        started.elapsed()
    );
}
