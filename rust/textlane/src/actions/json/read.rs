use serde::Deserialize;
use serde_json::Value;
use textlane_core::Error;

// Core's trees nest at most 1000 nodes, each an object and its `children` array; a deeper
// value would overflow the stack when dropped or written, which no error can catch
const MAX_DEPTH: usize = 2048;

/// Read a JSON document nested up to a tree's depth, as a binding receives a tree.
pub(crate) fn read_json(text: &str) -> Result<Value, Error> {
    if measure_depth(text) > MAX_DEPTH {
        return Err(Error::new(
            "tree-too-deep",
            format!("JSON nests at most {MAX_DEPTH} levels"),
        ));
    }
    let mut reader = serde_json::Deserializer::from_str(text);
    // serde_json stops at 128 levels, which a tree reaches at ~60 nested nodes
    reader.disable_recursion_limit();
    let value = Value::deserialize(serde_stacker::Deserializer::new(&mut reader));
    let value = value.and_then(|value| reader.end().map(|()| value));
    value.map_err(|error| Error::new("invalid-json", error.to_string()))
}

fn measure_depth(text: &str) -> usize {
    let (mut depth, mut deepest) = (0usize, 0usize);
    let (mut quoted, mut escaped) = (false, false);
    for byte in text.bytes() {
        match (quoted, byte) {
            (true, _) if escaped => escaped = false,
            (true, b'\\') => escaped = true,
            (true, b'"') | (false, b'"') => quoted = !quoted,
            (false, b'[' | b'{') => {
                depth += 1;
                deepest = deepest.max(depth);
            }
            (false, b']' | b'}') => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    deepest
}

#[cfg(test)]
#[path = "read_unit.rs"]
mod unit;
