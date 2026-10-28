use serde_json::Value;

use crate::Allocator;
use crate::models::error::Error;
use crate::models::position::{Position, UnistPoint, UnistPosition};

/// Read a node's `position`, kept as the unist position it is: it points into no source here.
pub fn deserialize_position<'a>(
    node: &Value,
    alloc: &'a Allocator,
) -> Result<Option<Position<'a>>, Error> {
    let Some(position) = node.get("position").filter(|value| !value.is_null()) else {
        return Ok(None);
    };
    let position = UnistPosition {
        start: deserialize_point(&position["start"])?,
        end: deserialize_point(&position["end"])?,
    };
    Ok(Some(Position::Unist(alloc.alloc(position))))
}

fn deserialize_point(point: &Value) -> Result<UnistPoint, Error> {
    let read = |key: &str| point.get(key).and_then(Value::as_u64).map(|n| n as usize);
    match (read("line"), read("column")) {
        (Some(line), Some(column)) => Ok(UnistPoint {
            line,
            column,
            offset: read("offset"),
        }),
        _ => Err(Error::new(
            "invalid-tree",
            format!("not a unist point: {point}"),
        )),
    }
}
