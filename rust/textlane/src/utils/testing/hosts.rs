use serde_json::{Value, json};
use textlane_core::{Error, File};

use crate::actions::message::read::read_messages;

/// Run a `host:<behaviour>[:<label>]` stand-in for a binding's plugin over a JSON tree, as
/// each binding's conformance runner does in its own language.
pub(crate) fn run_host(
    name: &str,
    options: &Value,
    mut tree: Value,
    file: &mut File,
) -> Result<Value, Error> {
    let text = |key: &str| options[key].as_str().unwrap_or_default().to_string();
    match read_behaviour(name) {
        "appendNode" => append_child(&mut tree, options["node"].clone()),
        "replaceTree" => return Ok(options["tree"].clone()),
        "message" => file.messages.extend(read_messages(&json!([options]))?),
        "countMessages" => {
            append_child(&mut tree, paragraph(&file.messages.len().to_string()))
        }
        "readValue" => {
            append_child(&mut tree, paragraph(file.value.as_deref().unwrap_or("")))
        }
        "fail" | "returnError" => {
            return Err(Error::new("plugin-failed", text("message")));
        }
        "recordOptions" => {}
        _ => {
            return Err(Error::new(
                "invalid-case",
                format!("no host behaviour `{name}`"),
            ));
        }
    }
    Ok(tree)
}

/// The behaviour a host plugin's name stands for, without its label.
pub(crate) fn read_behaviour(name: &str) -> &str {
    let rest = name.strip_prefix("host:").unwrap_or(name);
    rest.split(':').next().unwrap_or(rest)
}

fn append_child(tree: &mut Value, node: Value) {
    if let Some(children) = tree.get_mut("children").and_then(Value::as_array_mut) {
        children.push(node);
    }
}

fn paragraph(value: &str) -> Value {
    json!({ "type": "paragraph", "children": [{ "type": "text", "value": value }] })
}
