use serde_json::Value;
use textlane_core::{Error, Message, UnistPoint, UnistPosition};

/// Read the messages a binding's plugins reported, each `{reason, ruleId?, source?, place?}`
/// as vfile-message has it, its place a point or a position.
pub fn read_messages(value: &Value) -> Result<Vec<Message>, Error> {
    let Value::Array(messages) = value else {
        return Err(invalid_message(value));
    };
    messages.iter().map(read_message).collect()
}

fn read_message(value: &Value) -> Result<Message, Error> {
    let text = |key: &str| value.get(key).and_then(Value::as_str).map(str::to_string);
    let reason = text("reason").ok_or_else(|| invalid_message(value))?;
    let place = match value.get("place") {
        None | Some(Value::Null) => None,
        Some(place) => Some(read_place(place).ok_or_else(|| invalid_message(value))?),
    };
    Ok(Message {
        reason,
        source: text("source"),
        rule: text("ruleId"),
        place,
    })
}

fn read_place(place: &Value) -> Option<UnistPosition> {
    match (place.get("start"), place.get("end")) {
        (Some(start), Some(end)) => Some(UnistPosition {
            start: read_point(start)?,
            end: read_point(end)?,
        }),
        _ => {
            let point = read_point(place)?;
            Some(UnistPosition {
                start: point,
                end: point,
            })
        }
    }
}

fn read_point(point: &Value) -> Option<UnistPoint> {
    let number = |key: &str| point.get(key).and_then(Value::as_u64).map(|n| n as usize);
    Some(UnistPoint {
        line: number("line")?,
        column: number("column")?,
        offset: number("offset"),
    })
}

fn invalid_message(value: &Value) -> Error {
    Error::new(
        "invalid-message",
        format!("Expected a message with a `reason`, not `{value}`"),
    )
}

#[cfg(test)]
#[path = "read_unit.rs"]
mod unit;
