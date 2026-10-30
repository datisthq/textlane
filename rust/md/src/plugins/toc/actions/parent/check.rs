use serde_json::Value;
use textlane_core::Data;
use textlane_core::mdast::Node;

use crate::models::options::{ParentCheck, ParentTest};
use crate::utils::serializer::state::read_node_type;

/// unist-util-is's test of `node` against `test`; fields compare with JavaScript's `===`,
/// so an object or array never equals what a node holds.
pub(crate) fn check_parent(test: &ParentTest, node: &Node<'_>) -> bool {
    match test {
        ParentTest::One(check) => passes_check(check, node),
        ParentTest::Any(checks) => checks.iter().any(|check| passes_check(check, node)),
    }
}

fn passes_check(check: &ParentCheck, node: &Node<'_>) -> bool {
    match check {
        ParentCheck::Kind(kind) => read_node_type(node) == kind,
        ParentCheck::Fields(fields) => fields
            .iter()
            .all(|(key, value)| equals_field(node, key, value)),
    }
}

// A field a node lacks is `undefined`, which no JSON value is
fn equals_field(node: &Node<'_>, key: &str, expected: &Value) -> bool {
    if key == "type" {
        return expected.as_str() == Some(read_node_type(node));
    }
    let actual = match (node, key) {
        (Node::List(list), "ordered") => Field::Boolean(list.ordered),
        (Node::List(list), "start") => list
            .start
            .map_or(Field::Null, |start| Field::Number(f64::from(start))),
        (Node::List(list), "spread") => {
            list.spread.map_or(Field::Missing, Field::Boolean)
        }
        (Node::ListItem(item), "spread") => {
            item.spread.map_or(Field::Missing, Field::Boolean)
        }
        (Node::ListItem(item), "checked") => {
            item.checked.map_or(Field::Null, Field::Boolean)
        }
        (Node::DefListDescription(description), "spread") => {
            Field::Boolean(description.spread)
        }
        (Node::FootnoteDefinition(definition), "identifier") => {
            Field::String(definition.identifier)
        }
        (Node::FootnoteDefinition(definition), "label") => {
            definition.label.map_or(Field::Null, Field::String)
        }
        (Node::ContainerDirective(directive), "name") => Field::String(directive.name),
        (Node::LeafDirective(directive), "name") => Field::String(directive.name),
        (Node::TextDirective(directive), "name") => Field::String(directive.name),
        (Node::Heading(heading), "depth") => Field::Number(f64::from(heading.depth)),
        (Node::MdxJsxFlowElement(element) | Node::MdxJsxTextElement(element), "name") => {
            element.name.map_or(Field::Null, Field::String)
        }
        (Node::Unknown(unknown), key) => unknown
            .fields
            .iter()
            .rev()
            .find(|(name, _)| *name == key)
            .map_or(Field::Missing, |(_, value)| match value {
                Data::Null => Field::Null,
                Data::Boolean(value) => Field::Boolean(*value),
                Data::Number(value) => Field::Number(*value),
                Data::String(value) => Field::String(value),
                Data::Array(_) | Data::Object(_) => Field::Missing,
            }),
        _ => Field::Missing,
    };
    match (actual, expected) {
        (Field::Null, Value::Null) => true,
        (Field::Boolean(actual), Value::Bool(expected)) => actual == *expected,
        (Field::Number(actual), Value::Number(expected)) => {
            expected.as_f64().is_some_and(|expected| actual == expected)
        }
        (Field::String(actual), Value::String(expected)) => actual == expected,
        _ => false,
    }
}

enum Field<'a> {
    Missing,
    Null,
    Boolean(bool),
    Number(f64),
    String(&'a str),
}
