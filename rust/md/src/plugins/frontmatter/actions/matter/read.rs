use serde_json::{Map, Value};
use textlane_core::Error;

use crate::utils::yaml::parse::parse_yaml;

/// A YAML frontmatter's data as vfile-matter stores it: a falsy value becomes `{}`.
pub(crate) fn read_matter(yaml: Option<&str>) -> Result<Value, Error> {
    let value = match yaml {
        Some(yaml) => parse_yaml(yaml)?,
        None => Value::Null,
    };
    let falsy = match &value {
        Value::Null | Value::Bool(false) => true,
        Value::Number(number) => number.as_f64() == Some(0.0),
        Value::String(text) => text.is_empty(),
        _ => false,
    };
    Ok(if falsy {
        Value::Object(Map::new())
    } else {
        value
    })
}
