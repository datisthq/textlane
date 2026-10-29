use super::{KEYS, KINDS, Key, Kind};

#[test]
fn finds_every_name() {
    for (index, text) in KINDS.iter().enumerate() {
        assert_eq!(Kind::find(text), Some(Kind(index as u32)));
    }
    for (index, text) in KEYS.iter().enumerate() {
        assert_eq!(Key::find(text), Some(Key(index as u32)));
    }
    assert_eq!(Kind::find("TSTypeAnnotation"), None);
    assert_eq!(Key::find("typeAnnotation"), None);
}
