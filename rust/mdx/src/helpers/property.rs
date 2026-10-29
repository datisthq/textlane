use std::borrow::Cow;

use crate::helpers::property_table::{
    BOOLEAN, COMMA_SEPARATED, HAST_TO_REACT, HTML, Info, SVG,
};

/// What property-information's `find` returns: the info of a known property, or one made
/// up for `data-*` and unknown names, which have no space.
pub(crate) struct Found {
    pub(crate) property: Cow<'static, str>,
    pub(crate) attribute: Cow<'static, str>,
    pub(crate) space: Option<&'static str>,
    pub(crate) boolean: bool,
    pub(crate) comma_separated: bool,
}

/// Find `name`, an attribute or property, in the `html` or `svg` schema.
pub(crate) fn find_property(svg: bool, name: &str) -> Found {
    let schema: &[Info] = if svg { &SVG } else { &HTML };
    let normal = name.to_lowercase();
    if let Ok(index) = schema.binary_search_by(|info| info.normal.cmp(normal.as_str())) {
        let info = &schema[index];
        return Found {
            property: Cow::Borrowed(info.property),
            attribute: Cow::Borrowed(info.attribute),
            space: info.space,
            boolean: info.kinds & BOOLEAN != 0,
            comma_separated: info.kinds & COMMA_SEPARATED != 0,
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
        space: None,
        boolean: false,
        comma_separated: false,
    }
}

/// property-information's `hastToReact`: the React spelling of a property, when it differs.
pub(crate) fn react_property(property: &str) -> Option<&'static str> {
    HAST_TO_REACT
        .binary_search_by(|(key, _)| key.cmp(&property))
        .ok()
        .map(|index| HAST_TO_REACT[index].1)
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
