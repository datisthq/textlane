use crate::helpers::entity_table::ENTITIES;

/// Decode character references as acorn-jsx reads JSX text and attribute strings: XHTML
/// entities and `&#…;`/`&#x…;` within ten characters of the `&`, with `normalize` turning
/// CRLF into LF as JSX text does.
pub(crate) fn decode_entities(value: &str, normalize: bool) -> String {
    let mut result = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(index) = rest.find(['&', '\r']) {
        result.push_str(&rest[..index]);
        rest = &rest[index..];
        if let Some(after) = rest.strip_prefix("\r\n")
            && normalize
        {
            result.push('\n');
            rest = after;
            continue;
        }
        if rest.starts_with('\r') {
            result.push('\r');
            rest = &rest[1..];
            continue;
        }
        match read_entity(&rest[1..]) {
            Some((entity, length)) => {
                result.push_str(&entity);
                rest = &rest[1 + length..];
            }
            None => {
                result.push('&');
                rest = &rest[1..];
            }
        }
    }
    result.push_str(rest);
    result
}

// acorn-jsx counts UTF-16 units and makes a code unit with `fromCharCode`, so a reference
// past U+FFFF wraps around instead of naming that character
fn read_entity(after: &str) -> Option<(String, usize)> {
    let mut units = 0;
    for (index, character) in after.char_indices() {
        if units >= 10 {
            return None;
        }
        units += character.len_utf16();
        if character != ';' {
            continue;
        }
        let name = &after[..index];
        let entity = if let Some(hex) = name.strip_prefix("#x") {
            decode_number(hex, 16)
        } else if let Some(decimal) = name.strip_prefix('#') {
            decode_number(decimal, 10)
        } else {
            ENTITIES
                .binary_search_by(|(key, _)| key.cmp(&name))
                .ok()
                .map(|found| ENTITIES[found].1.to_string())
        };
        return entity.map(|entity| (entity, index + 1));
    }
    None
}

fn decode_number(digits: &str, radix: u32) -> Option<String> {
    if digits.is_empty() || !digits.chars().all(|digit| digit.is_digit(radix)) {
        return None;
    }
    let mut unit: u32 = 0;
    for digit in digits.chars() {
        unit = (unit.wrapping_mul(radix) + digit.to_digit(radix)?) & 0xffff;
    }
    Some(char::from_u32(unit).unwrap_or('\u{fffd}').to_string())
}
