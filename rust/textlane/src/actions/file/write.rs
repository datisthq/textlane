use serde_json::{Value, json};
use textlane_core::File;

use crate::actions::message::write::write_messages;

/// The file a binding's `process` hands back, shaped as a vfile: `{value, messages, data}`,
/// each message `{reason, ruleId, source, place}` in the order plugins reported them.
pub fn write_file(file: &File, value: &str) -> Value {
    json!({ "value": value, "messages": write_messages(&file.messages), "data": file.data })
}

#[cfg(test)]
#[path = "write_unit.rs"]
mod unit;
