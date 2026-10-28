use crate::helpers::entities::decode_named;

// The references HTML decodes without a semicolon, sorted
const LEGACY: [&str; 106] = [
    "AElig", "AMP", "Aacute", "Acirc", "Agrave", "Aring", "Atilde", "Auml", "COPY",
    "Ccedil", "ETH", "Eacute", "Ecirc", "Egrave", "Euml", "GT", "Iacute", "Icirc",
    "Igrave", "Iuml", "LT", "Ntilde", "Oacute", "Ocirc", "Ograve", "Oslash", "Otilde",
    "Ouml", "QUOT", "REG", "THORN", "Uacute", "Ucirc", "Ugrave", "Uuml", "Yacute",
    "aacute", "acirc", "acute", "aelig", "agrave", "amp", "aring", "atilde", "auml",
    "brvbar", "ccedil", "cedil", "cent", "copy", "curren", "deg", "divide", "eacute",
    "ecirc", "egrave", "eth", "euml", "frac12", "frac14", "frac34", "gt", "iacute",
    "icirc", "iexcl", "igrave", "iquest", "iuml", "laquo", "lt", "macr", "micro",
    "middot", "nbsp", "not", "ntilde", "oacute", "ocirc", "ograve", "ordf", "ordm",
    "oslash", "otilde", "ouml", "para", "plusmn", "pound", "quot", "raquo", "reg",
    "sect", "shy", "sup1", "sup2", "sup3", "szlig", "thorn", "times", "uacute", "ucirc",
    "ugrave", "uml", "uuml", "yacute", "yen", "yuml",
];

// Windows-1252's characters, which HTML reads numeric references to C1 controls as
const INVALID: [(u32, char); 28] = [
    (0, '\u{FFFD}'),
    (128, '€'),
    (130, '‚'),
    (131, 'ƒ'),
    (132, '„'),
    (133, '…'),
    (134, '†'),
    (135, '‡'),
    (136, 'ˆ'),
    (137, '‰'),
    (138, 'Š'),
    (139, '‹'),
    (140, 'Œ'),
    (142, 'Ž'),
    (145, '‘'),
    (146, '’'),
    (147, '“'),
    (148, '”'),
    (149, '•'),
    (150, '–'),
    (151, '—'),
    (152, '˜'),
    (153, '™'),
    (154, 'š'),
    (155, '›'),
    (156, 'œ'),
    (158, 'ž'),
    (159, 'Ÿ'),
];

/// Decode the character references in an attribute value, as parse-entities does with
/// `attribute: true`: a reference without a semicolon stays when a letter, digit or `=`
/// follows it.
pub(crate) fn decode_attribute(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut result = String::with_capacity(value.len());
    let mut index = 0;
    let mut start = 0;
    while index < bytes.len() {
        if bytes[index] != b'&' {
            index += 1;
            continue;
        }
        if matches!(
            bytes.get(index + 1),
            None | Some(b'\t' | b'\n' | b'\x0C' | b' ' | b'&' | b'<')
        ) {
            index += 1;
            continue;
        }
        match decode_reference(value, index + 1) {
            Some((reference, end)) => {
                result.push_str(&value[start..index]);
                result.push_str(&reference);
                index = end;
                start = end;
            }
            None => index += 1,
        }
    }
    result.push_str(&value[start..]);
    result
}

// The decoded reference that starts after the `&` at `start`, and where it ends
fn decode_reference(value: &str, start: usize) -> Option<(String, usize)> {
    let bytes = value.as_bytes();
    let (numeric, radix, begin) = match (bytes.get(start), bytes.get(start + 1)) {
        (Some(b'#'), Some(b'x' | b'X')) => (true, 16, start + 2),
        (Some(b'#'), _) => (true, 10, start + 1),
        _ => (false, 0, start),
    };
    let accept = |byte: u8| match radix {
        16 => byte.is_ascii_hexdigit(),
        10 => byte.is_ascii_digit(),
        _ => byte.is_ascii_alphanumeric(),
    };
    let mut end = begin;
    let mut legacy_end = None;
    while end < bytes.len() && accept(bytes[end]) {
        end += 1;
        if !numeric && LEGACY.binary_search(&&value[begin..end]).is_ok() {
            legacy_end = Some(end);
        }
    }
    let characters = &value[begin..end];
    if characters.is_empty() {
        return None;
    }
    let terminated = bytes.get(end) == Some(&b';');
    if numeric {
        let code = u32::from_str_radix(characters, radix).unwrap_or(u32::MAX);
        let after = end + usize::from(terminated);
        return Some((decode_numeric(code).to_string(), after));
    }
    if terminated && let Some(decoded) = decode_named(characters) {
        return Some((decoded.to_string(), end + 1));
    }
    let legacy_end = legacy_end?;
    let following = bytes.get(legacy_end).copied();
    if following.is_some_and(|byte| byte == b'=' || byte.is_ascii_alphanumeric()) {
        return None;
    }
    let decoded = decode_named(&value[begin..legacy_end])?;
    Some((decoded.to_string(), legacy_end))
}

fn decode_numeric(code: u32) -> char {
    if (0xD800..=0xDFFF).contains(&code) || code > 0x10FFFF {
        return '\u{FFFD}';
    }
    if let Ok(index) = INVALID.binary_search_by_key(&code, |(key, _)| *key) {
        return INVALID[index].1;
    }
    char::from_u32(code).unwrap_or('\u{FFFD}')
}
