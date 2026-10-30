/// Text as hast-util-to-html writes it: `<` and `&` as hexadecimal references.
pub(crate) fn push_text(output: &mut String, value: &str) {
    let mut start = 0;
    for index in memchr::memchr2_iter(b'<', b'&', value.as_bytes()) {
        output.push_str(&value[start..index]);
        output.push_str(if value.as_bytes()[index] == b'<' {
            "&#x3C;"
        } else {
            "&#x26;"
        });
        start = index + 1;
    }
    output.push_str(&value[start..]);
}

/// A double-quoted attribute value as hast-util-to-html writes it by default.
pub(crate) fn push_attribute(output: &mut String, value: &str) {
    push_escaped(output, value, |byte| {
        matches!(byte, b'\0' | b'"' | b'&' | b'\'' | b'`')
    });
}

// stringify-entities: each matched byte, all ASCII, as a hexadecimal reference
fn push_escaped(output: &mut String, value: &str, matches: impl Fn(u8) -> bool) {
    let bytes = value.as_bytes();
    let mut start = 0;
    for (index, &byte) in bytes.iter().enumerate() {
        if !matches(byte) {
            continue;
        }
        output.push_str(&value[start..index]);
        output.push_str("&#x");
        if byte >= 16 {
            output.push(char::from(HEX[usize::from(byte >> 4)]));
        }
        output.push(char::from(HEX[usize::from(byte & 15)]));
        output.push(';');
        start = index + 1;
    }
    output.push_str(&value[start..]);
}

const HEX: &[u8; 16] = b"0123456789ABCDEF";
