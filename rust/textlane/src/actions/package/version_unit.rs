use super::*;

#[test]
fn read_version_matches_manifest() {
    let manifest = include_str!("../../../../../version.txt");
    assert_eq!(read_version(), manifest.trim());
}
