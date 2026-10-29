use serde_json::{Map, Value};

use crate::actions::hast::serialize::serialize_data;
use crate::actions::position::serialize::serialize_position;
use crate::models::lines::Lines;
use crate::models::mdx::{MdxJsxAttribute, MdxJsxAttributeValue};

/// Write a JSX element's `name` and `attributes`, as mdast-util-mdx-jsx lists them.
pub(crate) fn insert_mdx_jsx(
    object: &mut Map<String, Value>,
    name: Option<&str>,
    attributes: &[MdxJsxAttribute<'_>],
    lines: Option<&Lines<'_>>,
) {
    object.insert("name".into(), name.into());
    let attributes = attributes
        .iter()
        .map(|attribute| serialize_attribute(attribute, lines))
        .collect();
    object.insert("attributes".into(), Value::Array(attributes));
}

fn serialize_attribute(
    attribute: &MdxJsxAttribute<'_>,
    lines: Option<&Lines<'_>>,
) -> Value {
    let mut object = Map::new();
    let (position, data) = match attribute {
        MdxJsxAttribute::Attribute {
            name,
            value,
            position,
            data,
        } => {
            object.insert("type".into(), "mdxJsxAttribute".into());
            object.insert("name".into(), (*name).into());
            let value = match value {
                None => Value::Null,
                Some(MdxJsxAttributeValue::Literal(value)) => (*value).into(),
                Some(MdxJsxAttributeValue::Expression { value, data }) => {
                    let mut expression = Map::new();
                    expression
                        .insert("type".into(), "mdxJsxAttributeValueExpression".into());
                    expression.insert("value".into(), (*value).into());
                    if let Some(data) = data {
                        expression.insert("data".into(), serialize_data(data));
                    }
                    Value::Object(expression)
                }
            };
            object.insert("value".into(), value);
            (position, data)
        }
        MdxJsxAttribute::Expression {
            value,
            position,
            data,
        } => {
            object.insert("type".into(), "mdxJsxExpressionAttribute".into());
            object.insert("value".into(), (*value).into());
            (position, data)
        }
    };
    if let Some(data) = data {
        object.insert("data".into(), serialize_data(data));
    }
    serialize_position(&mut object, position.as_ref(), lines);
    Value::Object(object)
}
