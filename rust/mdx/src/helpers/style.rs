/// style-to-js 1.1.21 with `reactCompat`: a `style` attribute's declarations as an object,
/// names camel-cased, a repeated name keeping its first place and its last value.
pub(crate) fn parse_style(value: &str) -> Result<Vec<(String, String)>, ()> {
    let mut result: Vec<(String, String)> = Vec::new();
    for (property, value) in parse_declarations(value)? {
        if property.is_empty() || value.is_empty() {
            continue;
        }
        let property = camel_case(&property);
        match result.iter_mut().find(|(name, _)| *name == property) {
            Some(entry) => entry.1 = value,
            None => result.push((property, value)),
        }
    }
    Ok(result)
}

// inline-style-parser's declarations: `property: value;`, comments between them skipped
fn parse_declarations(style: &str) -> Result<Vec<(String, String)>, ()> {
    let mut declarations = Vec::new();
    let mut rest = skip_comments(style);
    loop {
        let property_length = property_length(rest);
        if property_length == 0 {
            return Ok(declarations);
        }
        let property = strip_comments(&rest[..property_length]);
        rest = skip_comments(rest[property_length..].trim_start());
        let Some(after) = rest.strip_prefix(':') else {
            return Err(());
        };
        rest = after.trim_start();
        let value_length = value_length(rest);
        let value = strip_comments(&rest[..value_length]);
        rest = rest[value_length..].trim_start_matches(|character: char| {
            character == ';' || character.is_whitespace()
        });
        declarations.push((property.trim().to_string(), value.trim().to_string()));
        rest = skip_comments(rest);
    }
}

// `/^(\*?[-#/*\\\w]+(\[[0-9a-z_-]+\])?)\s*/`
fn property_length(value: &str) -> usize {
    let bytes = value.as_bytes();
    let mut index = usize::from(bytes.first() == Some(&b'*'));
    let start = index;
    while bytes.get(index).is_some_and(|byte| {
        byte.is_ascii_alphanumeric()
            || matches!(byte, b'-' | b'#' | b'/' | b'*' | b'\\' | b'_')
    }) {
        index += 1;
    }
    if index == start {
        return 0;
    }
    if bytes.get(index) == Some(&b'[') {
        let mut end = index + 1;
        while bytes.get(end).is_some_and(|byte| {
            byte.is_ascii_digit()
                || byte.is_ascii_lowercase()
                || matches!(byte, b'_' | b'-')
        }) {
            end += 1;
        }
        if end > index + 1 && bytes.get(end) == Some(&b']') {
            index = end + 1;
        }
    }
    index
}

// `/^((?:'(?:\\'|.)*?'|"(?:\\"|.)*?"|\([^)]*?\)|[^};])+)/`
fn value_length(value: &str) -> usize {
    let bytes = value.as_bytes();
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'\'' | b'"' => match closing_quote(&bytes[index + 1..], byte) {
                Some(length) => index += length + 2,
                None => index += 1,
            },
            b'(' => match bytes[index + 1..].iter().position(|byte| *byte == b')') {
                Some(length) => index += length + 2,
                None => index += 1,
            },
            b'}' | b';' => break,
            _ => index += 1,
        }
    }
    while !value.is_char_boundary(index) {
        index += 1;
    }
    index
}

fn closing_quote(bytes: &[u8], quote: u8) -> Option<usize> {
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'\\' if bytes.get(index + 1) == Some(&quote) => index += 2,
            b'\n' | b'\r' => return None,
            _ if byte == quote => return Some(index),
            _ => index += 1,
        }
    }
    None
}

fn skip_comments(value: &str) -> &str {
    let mut rest = value.trim_start();
    while let Some(after) = rest.strip_prefix("/*") {
        match after.find("*/") {
            Some(end) => rest = after[end + 2..].trim_start(),
            None => return rest,
        }
    }
    rest
}

fn strip_comments(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(start) = rest.find("/*") {
        match rest[start + 2..].find("*/") {
            Some(end) => {
                result.push_str(&rest[..start]);
                rest = &rest[start + 2 + end + 2..];
            }
            None => break,
        }
    }
    result.push_str(rest);
    result
}

// style-to-js's `camelCase`: custom properties and names without dashes stay
fn camel_case(property: &str) -> String {
    if !property.contains('-')
        || (property.starts_with("--")
            && property[2..]
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
            && property.len() > 2)
    {
        return property.to_string();
    }
    let mut property = property.to_lowercase();
    if property.starts_with("-ms-") {
        property.remove(0);
    }
    let mut result = String::with_capacity(property.len());
    let mut characters = property.chars().peekable();
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
