use serde_json::Value;

use crate::actions::hast::deserialize::deserialize_data;
use crate::actions::position::deserialize::deserialize_position;
use crate::models::data::Data;
use crate::models::error::Error;
use crate::models::mdx::{MdxJsxAttribute, MdxJsxAttributeValue};
use crate::{Allocator, ArenaVec};

/// Read a JSX element's `name`; a missing or null one makes a fragment.
pub(crate) fn read_mdx_jsx_name<'a>(
    value: &Value,
    alloc: &'a Allocator,
) -> Option<&'a str> {
    value
        .get("name")
        .and_then(Value::as_str)
        .map(|name| &*alloc.alloc_str(name))
}

/// Read a JSX element's `attributes`, an empty list when missing.
pub(crate) fn read_mdx_jsx_attributes<'a>(
    value: &Value,
    alloc: &'a Allocator,
    depth: usize,
) -> Result<ArenaVec<'a, MdxJsxAttribute<'a>>, Error> {
    let mut attributes = ArenaVec::new_in(alloc);
    for item in value
        .get("attributes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let position = deserialize_position(item, alloc)?;
        let data = read_data(item, alloc, depth)?;
        let read_str = |item: &Value| -> &'a str {
            alloc.alloc_str(item.get("value").and_then(Value::as_str).unwrap_or(""))
        };
        let attribute = match item.get("type").and_then(Value::as_str) {
            Some("mdxJsxExpressionAttribute") => MdxJsxAttribute::Expression {
                value: read_str(item),
                position,
                data,
            },
            Some("mdxJsxAttribute") => MdxJsxAttribute::Attribute {
                name: alloc
                    .alloc_str(item.get("name").and_then(Value::as_str).unwrap_or("")),
                value: match item.get("value") {
                    Some(Value::String(value)) => {
                        Some(MdxJsxAttributeValue::Literal(alloc.alloc_str(value)))
                    }
                    Some(expression @ Value::Object(_)) => {
                        Some(MdxJsxAttributeValue::Expression {
                            value: read_str(expression),
                            data: read_data(expression, alloc, depth)?,
                        })
                    }
                    _ => None,
                },
                position,
                data,
            },
            _ => {
                return Err(Error::new(
                    "invalid-tree",
                    format!("not a JSX attribute: {item}"),
                ));
            }
        };
        attributes.push(attribute);
    }
    Ok(attributes)
}

/// Read the `value` of an MDX expression or ESM node, empty when missing.
pub(crate) fn read_mdx_value<'a>(value: &Value, alloc: &'a Allocator) -> &'a str {
    alloc.alloc_str(value.get("value").and_then(Value::as_str).unwrap_or(""))
}

fn read_data<'a>(
    value: &Value,
    alloc: &'a Allocator,
    depth: usize,
) -> Result<Option<&'a Data<'a>>, Error> {
    match value.get("data") {
        Some(data) => Ok(Some(&*alloc.alloc(deserialize_data(
            data,
            alloc,
            depth + 2,
        )?))),
        None => Ok(None),
    }
}
