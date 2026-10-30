use serde_json::Value;

use super::describe_plugins;

fn find_upstream<'a>(described: &'a Value, domain: &str, name: &str) -> &'a Value {
    let plugins = described[domain]["plugins"].as_array().map(Vec::as_slice);
    let plugin = plugins
        .unwrap_or_default()
        .iter()
        .find(|plugin| plugin["name"] == name);
    plugin.map_or(&Value::Null, |plugin| &plugin["upstream"])
}

#[test]
fn lists_each_plugins_upstream() {
    let described = describe_plugins();
    assert_eq!(find_upstream(&described, "md", "md.gfm"), "remark-gfm");
    assert_eq!(
        find_upstream(&described, "html", "html.sanitize"),
        "rehype-sanitize"
    );
}
