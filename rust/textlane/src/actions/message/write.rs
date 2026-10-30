use serde_json::{Value, json};
use textlane_core::{Message, UnistPoint};

/// Messages as a vfile holds them, each `{reason, ruleId, source, place}`, in order.
pub fn write_messages(messages: &[Message]) -> Value {
    let messages = messages.iter().map(|message| {
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
    Value::Array(messages.collect())
}

fn write_point(point: &UnistPoint) -> Value {
    json!({ "line": point.line, "column": point.column, "offset": point.offset })
}
