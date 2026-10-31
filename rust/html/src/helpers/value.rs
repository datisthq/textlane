use std::borrow::Cow;

use serde_json::Value;
use textlane_core::hast::{Element, Property, PropertyItem, PropertyValue};
use textlane_core::{Allocator, ArenaVec};

use crate::actions::hast::stringify::write_js_number;

/// The value of an element's property, by its hast name.
pub(crate) fn find_value<'n, 'a>(
    element: &'n Element<'a>,
    name: &str,
) -> Option<&'n PropertyValue<'a>> {
    element
        .properties
        .iter()
        .rev()
        .find(|property| property.name == name)
        .map(|property| &property.value)
}

/// Whether JavaScript reads a property as true: set, and not `false`, `0`, `NaN` or `''`.
pub(crate) fn is_truthy(value: Option<&PropertyValue<'_>>) -> bool {
    match value {
        None | Some(PropertyValue::Null) => false,
        Some(PropertyValue::Boolean(value)) => *value,
        Some(PropertyValue::Number(value)) => *value != 0.0 && !value.is_nan(),
        Some(PropertyValue::String(value)) => !value.is_empty(),
        Some(PropertyValue::List(_)) => true,
    }
}

/// A property as JavaScript's `String(value)` writes it.
pub(crate) fn write_value<'a>(value: &PropertyValue<'a>) -> Cow<'a, str> {
    match value {
        PropertyValue::Null => Cow::Borrowed("null"),
        PropertyValue::Boolean(value) => {
            Cow::Borrowed(if *value { "true" } else { "false" })
        }
        PropertyValue::Number(value) => Cow::Owned(write_js_number(*value)),
        PropertyValue::String(value) => Cow::Borrowed(value),
        PropertyValue::List(items) => {
            let items: Vec<Cow<'a, str>> = items
                .iter()
                .map(|item| match item {
                    PropertyItem::Number(value) => Cow::Owned(write_js_number(*value)),
                    PropertyItem::String(value) => Cow::Borrowed(*value),
                })
                .collect();
            Cow::Owned(items.join(","))
        }
    }
}

/// `String(properties[name] || '')`: a property's text, empty where it is unset or false.
pub(crate) fn read_value<'a>(element: &Element<'a>, name: &str) -> Cow<'a, str> {
    let value = find_value(element, name);
    match value {
        Some(value) if is_truthy(Some(value)) => write_value(value),
        _ => Cow::Borrowed(""),
    }
}

/// A property that is a string, compared as hast holds it.
pub(crate) fn find_string<'a>(element: &Element<'a>, name: &str) -> Option<&'a str> {
    match find_value(element, name) {
        Some(PropertyValue::String(value)) => Some(value),
        _ => None,
    }
}

/// JavaScript's `Number.parseInt(value, 10)`, `None` where it is `NaN`.
pub(crate) fn parse_int(value: &str) -> Option<f64> {
    let trimmed = value.trim_start_matches(|character: char| {
        character.is_whitespace() || character == '\u{FEFF}'
    });
    let (negative, digits) = match trimmed.as_bytes().first() {
        Some(b'-') => (true, &trimmed[1..]),
        Some(b'+') => (false, &trimmed[1..]),
        _ => (false, trimmed),
    };
    let digits = digits
        .bytes()
        .take_while(u8::is_ascii_digit)
        .map(|digit| f64::from(digit - b'0'));
    let mut number = None;
    for digit in digits {
        number = Some(number.unwrap_or(0.0) * 10.0 + digit);
    }
    number.map(|number| if negative { -number } else { number })
}

/// Set a property, replacing one of that name.
pub(crate) fn set_property<'a>(
    element: &mut Element<'a>,
    name: &'a str,
    value: PropertyValue<'a>,
) {
    match element
        .properties
        .iter_mut()
        .find(|property| property.name == name)
    {
        Some(property) => property.value = value,
        None => element.properties.push(Property { name, value }),
    }
}

/// A JSON option's value as a hast property holds it; objects and `null` are `null`.
pub(crate) fn read_json_value<'a>(
    value: &Value,
    alloc: &'a Allocator,
) -> PropertyValue<'a> {
    match value {
        Value::Bool(value) => PropertyValue::Boolean(*value),
        Value::Number(number) => {
            PropertyValue::Number(number.as_f64().unwrap_or_default())
        }
        Value::String(text) => PropertyValue::String(alloc.alloc_str(text)),
        Value::Array(items) => {
            let mut list = ArenaVec::with_capacity_in(items.len(), alloc);
            for item in items {
                match item {
                    Value::Number(number) => {
                        list.push(PropertyItem::Number(
                            number.as_f64().unwrap_or_default(),
                        ));
                    }
                    Value::String(text) => {
                        list.push(PropertyItem::String(alloc.alloc_str(text)))
                    }
                    _ => {}
                }
            }
            PropertyValue::List(list)
        }
        Value::Null | Value::Object(_) => PropertyValue::Null,
    }
}
