use serde_json::{Map, Value, json};

use crate::models::lines::Lines;
use crate::models::position::{Position, UnistPoint};

/// Write a node's `position` into its JSON object; a span needs the source's lines.
pub fn serialize_position(
    object: &mut Map<String, Value>,
    position: Option<&Position<'_>>,
    lines: Option<&Lines<'_>>,
) {
    let position = match (position, lines) {
        (Some(position @ Position::Span { .. }), Some(lines)) => {
            lines.to_unist_position(position)
        }
        (Some(Position::Unist(position)), _) => (*position).clone(),
        _ => return,
    };
    object.insert(
        "position".into(),
        json!({
            "start": serialize_point(&position.start),
            "end": serialize_point(&position.end),
        }),
    );
}

fn serialize_point(point: &UnistPoint) -> Value {
    let mut object = Map::new();
    object.insert("line".into(), point.line.into());
    object.insert("column".into(), point.column.into());
    if let Some(offset) = point.offset {
        object.insert("offset".into(), offset.into());
    }
    Value::Object(object)
}
