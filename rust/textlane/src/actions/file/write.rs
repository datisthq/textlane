use serde_json::{Value, json};
use textlane_core::{File, UnistPoint};

/// The file a binding's `process` hands back, shaped as a vfile: `{value, messages, data}`,
/// each message `{reason, ruleId, source, place}` in the order plugins reported them.
pub fn write_file(file: &File, value: &str) -> Value {
    let messages = file.messages.iter().map(|message| {
        json!({
            "reason": message.reason,
            "ruleId": message.rule,
            "source": message.source,
            "place": message.place.as_ref().map(|place| json!({
                "start": write_point(&place.start),
                "end": write_point(&place.end),
            })),
        })
    });
    json!({ "value": value, "messages": messages.collect::<Vec<_>>(), "data": {} })
}

fn write_point(point: &UnistPoint) -> Value {
    json!({ "line": point.line, "column": point.column, "offset": point.offset })
}

#[cfg(test)]
#[path = "write_unit.rs"]
mod unit;
