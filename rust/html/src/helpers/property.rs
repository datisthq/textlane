use std::borrow::Cow;

use crate::helpers::schema::{
    BOOLEAN, COMMA_OR_SPACE_SEPARATED, COMMA_SEPARATED, Info, NUMBER, OVERLOADED_BOOLEAN,
    SPACE_SEPARATED,
};

/// What property-information's `find` returns: the info of a known property, or one made
/// up for `data-*` and unknown names.
#[derive(Clone, Debug)]
pub(crate) struct Found {
    pub(crate) property: Cow<'static, str>,
    pub(crate) attribute: Cow<'static, str>,
    pub(crate) kinds: u16,
}

/// Find `name`, an attribute or property, in a schema, as property-information's `find`.
pub(crate) fn find_property(schema: &'static [Info], name: &str) -> Found {
    let normal = name.to_lowercase();
    if let Ok(index) = schema.binary_search_by(|info| info.normal.cmp(normal.as_str())) {
        let info = &schema[index];
        return Found {
            property: Cow::Borrowed(info.property),
            attribute: Cow::Borrowed(info.attribute),
            kinds: info.kinds,
        };
    }
    let mut property = name.to_string();
    let mut attribute = name.to_string();
    if normal.len() > 4 && normal.starts_with("data") && is_valid_data(name) {
        if name.as_bytes()[4] == b'-' {
            let rest = camelcase_dashes(&name[5..]);
            let mut characters = rest.chars();
            property = match characters.next() {
                Some(first) => {
                    format!("data{}{}", first.to_uppercase(), characters.as_str())
                }
                None => "data".to_string(),
            };
        } else {
            let rest = &name[4..];
            if !has_dash_letter(rest) {
                let mut dashes = kebab_capitals(rest);
                if !dashes.starts_with('-') {
                    dashes.insert(0, '-');
                }
                attribute = format!("data{dashes}");
            }
        }
    }
    Found {
        property: Cow::Owned(property),
        attribute: Cow::Owned(attribute),
        kinds: 0,
    }
}

// `/^data[-\w.:]+$/i`
fn is_valid_data(name: &str) -> bool {
    name.len() > 4
        && name[4..].bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':')
        })
}

// `-a` becomes `A`
fn camelcase_dashes(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '-'
            && let Some(next) =
                characters.peek().copied().filter(char::is_ascii_lowercase)
        {
            result.push(next.to_ascii_uppercase());
            characters.next();
        } else {
            result.push(character);
        }
    }
    result
}

fn has_dash_letter(value: &str) -> bool {
    value
        .as_bytes()
        .windows(2)
        .any(|pair| pair[0] == b'-' && pair[1].is_ascii_lowercase())
}

fn kebab_capitals(value: &str) -> String {
    let mut result = String::with_capacity(value.len() + 4);
    for character in value.chars() {
        if character.is_ascii_uppercase() {
            result.push('-');
            result.push(character.to_ascii_lowercase());
        } else {
            result.push(character);
        }
    }
    result
}

/// A property value as hastscript makes it from an attribute's string.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Value {
    Boolean(bool),
    Number(f64),
    String(String),
    List(Vec<Item>),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Item {
    Number(f64),
    String(String),
}

/// Parse an attribute value as hastscript's `addProperty` does for a string.
pub(crate) fn parse_property_value(found: &Found, value: &str) -> Value {
    let list = if found.kinds & SPACE_SEPARATED != 0 {
        Some(split_spaces(value))
    } else if found.kinds & COMMA_SEPARATED != 0 {
        Some(split_commas(value))
    } else if found.kinds & COMMA_OR_SPACE_SEPARATED != 0 {
        Some(split_spaces(&split_commas(value).join(" ")))
    } else {
        None
    };
    match list {
        Some(list) => Value::List(
            list.into_iter()
                .map(|item| match parse_primitive(found, item) {
                    Value::Number(number) => Item::Number(number),
                    Value::Boolean(_) => Item::String("true".to_string()),
                    Value::String(value) => Item::String(value),
                    Value::List(_) => Item::String(String::new()),
                })
                .collect(),
        ),
        None => parse_primitive(found, value.to_string()),
    }
}

fn parse_primitive(found: &Found, value: String) -> Value {
    if found.kinds & NUMBER != 0
        && !value.is_empty()
        && let Some(number) = parse_js_number(&value)
    {
        return Value::Number(number);
    }
    if found.kinds & (BOOLEAN | OVERLOADED_BOOLEAN) != 0
        && (value.is_empty() || value.to_lowercase() == found.property.to_lowercase())
    {
        return Value::Boolean(true);
    }
    Value::String(value)
}

// space-separated-tokens' `parse`
fn split_spaces(value: &str) -> Vec<String> {
    value
        .split([' ', '\t', '\n', '\r', '\u{C}'])
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect()
}

// comma-separated-tokens' `parse`: a trailing empty token is dropped, inner ones are kept
fn split_commas(value: &str) -> Vec<String> {
    let parts: Vec<&str> = value.split(',').collect();
    let last = parts.len() - 1;
    parts
        .into_iter()
        .enumerate()
        .map(|(index, part)| (index, trim_js(part)))
        .filter(|(index, token)| !token.is_empty() || *index != last)
        .map(|(_, token)| token.to_string())
        .collect()
}

/// JavaScript's `Number(value)`, `None` where it is `NaN`.
pub(crate) fn parse_js_number(value: &str) -> Option<f64> {
    let trimmed = trim_js(value);
    if trimmed.is_empty() {
        return Some(0.0);
    }
    let radix = |prefix: &[&str], radix: u32| {
        prefix
            .iter()
            .find_map(|prefix| trimmed.strip_prefix(prefix))
            .map(|digits| {
                if digits.is_empty() {
                    return None;
                }
                u128::from_str_radix(digits, radix)
                    .ok()
                    .map(|number| number as f64)
            })
    };
    if let Some(number) = radix(&["0x", "0X"], 16)
        .or_else(|| radix(&["0o", "0O"], 8))
        .or_else(|| radix(&["0b", "0B"], 2))
    {
        return number;
    }
    let unsigned = trimmed.strip_prefix(['+', '-']).unwrap_or(trimmed);
    if unsigned == "Infinity" {
        return Some(if trimmed.starts_with('-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        });
    }
    let bytes = unsigned.as_bytes();
    let valid = !bytes.is_empty()
        && bytes.iter().all(|byte| {
            byte.is_ascii_digit() || matches!(byte, b'.' | b'e' | b'E' | b'+' | b'-')
        })
        && bytes.iter().any(u8::is_ascii_digit)
        && !unsigned.starts_with(['e', 'E']);
    if !valid {
        return None;
    }
    trimmed.parse::<f64>().ok()
}

// JavaScript's `String.prototype.trim`, which also trims `\u{FEFF}` and Unicode spaces
fn trim_js(value: &str) -> &str {
    value.trim_matches(|character: char| {
        character.is_whitespace() || character == '\u{FEFF}'
    })
}
