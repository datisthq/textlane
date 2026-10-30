use super::list_plugins;
use crate::actions::plugin::find::find_plugin;

#[test]
fn lists_every_public_plugin_once() {
    let names = list_plugins();
    assert!(names.contains(&"md.toHtml"));
    assert!(names.contains(&"nl.indefiniteArticle"));
    let mut unique = names.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), names.len());
}

#[test]
fn lists_no_test_plugin_and_only_buildable_ones() {
    for name in list_plugins() {
        assert!(!name.starts_with("test."));
        assert!(find_plugin(name).is_some(), "{name}");
    }
}
