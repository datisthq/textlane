use crate::helpers::reference::{
    accept_reference_byte, decode_reference, measure_reference,
};

/// Decode the character escapes and references in a raw string, as micromark's
/// `decodeString` does; an unknown reference stays as it is.
pub(crate) fn decode_string(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut result = String::with_capacity(value.len());
    let mut start = 0;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' if bytes.get(index + 1).is_some_and(u8::is_ascii_punctuation) => {
                result.push_str(&value[start..index]);
                start = index + 1;
                index += 2;
            }
            b'&' => {
                let (marker, value_start) =
                    match (bytes.get(index + 1), bytes.get(index + 2)) {
                        (Some(b'#'), Some(b'x' | b'X')) => (b'x', index + 3),
                        (Some(b'#'), _) => (b'#', index + 2),
                        _ => (b'&', index + 1),
                    };
                let mut end = value_start;
                while end < bytes.len()
                    && end - value_start < measure_reference(marker)
                    && accept_reference_byte(marker, bytes[end])
                {
                    end += 1;
                }
                let decoded = (end > value_start && bytes.get(end) == Some(&b';'))
                    .then(|| decode_reference(&value[value_start..end], marker))
                    .flatten();
                match decoded {
                    Some(decoded) => {
                        result.push_str(&value[start..index]);
                        result.push_str(&decoded);
                        index = end + 1;
                        start = index;
                    }
                    None => index += 1,
                }
            }
            _ => index += 1,
        }
    }
    result.push_str(&value[start..]);
    result
}
