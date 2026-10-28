use super::*;

#[test]
fn read_version_matches_manifest() {
    assert_eq!(read_version(), "0.0.0");
}
