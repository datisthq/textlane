use super::*;

#[test]
fn maps_bytes_and_units_both_ways() {
    let location = Location::new("aé😀\nb");
    assert_eq!(location.len(), 6);
    assert_eq!(location.to_unit(3), 2);
    assert_eq!(location.to_unit(2), 1);
    assert_eq!(location.to_byte(3), None);
    assert_eq!(location.to_byte(4), Some(7));
    assert_eq!(location.to_byte(6), Some(9));
    assert_eq!(location.to_byte(7), None);
}

#[test]
fn reads_points_as_vfile_location_does() {
    let location = Location::new("ab\r\ncd\re");
    let point = |line, column, offset| UnistPoint {
        line,
        column,
        offset: Some(offset),
    };
    assert_eq!(location.to_point(3), Some(point(1, 4, 3)));
    assert_eq!(location.to_point(4), Some(point(2, 1, 4)));
    assert_eq!(location.to_point(8), Some(point(3, 2, 8)));
    assert_eq!(location.to_point(9), None);
    assert_eq!(location.to_offset(2, 2), Some(5));
    assert_eq!(location.to_offset(2, 4), None);
    assert_eq!(location.to_offset(3, 2), Some(8));
    assert_eq!(location.to_offset(4, 1), None);
}

#[test]
fn places_a_unit_inside_a_pair_by_unist_points() {
    let alloc = Allocator::new();
    let location = Location::new("😀a");
    assert_eq!(
        location.to_position(0, 2, &alloc),
        Some(Position::span(Point::new(0, 0), Point::new(4, 0)))
    );
    let inside = location.to_position(1, 3, &alloc);
    assert!(
        matches!(inside, Some(Position::Unist(position)) if position.start.column == 2)
    );
    assert_eq!(location.to_position(0, 4, &alloc), None);
}
