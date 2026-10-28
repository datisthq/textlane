/// Percent-encode what a URL may not hold, as micromark's `normalizeUri` does; an
/// existing `%` escape of two alphanumerics is kept.
pub(crate) fn normalize_uri(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut chars = value.char_indices().peekable();
    while let Some((index, character)) = chars.next() {
        let keep = match character {
            '%' => {
                bytes.get(index + 1).is_some_and(u8::is_ascii_alphanumeric)
                    && bytes.get(index + 2).is_some_and(u8::is_ascii_alphanumeric)
            }
            '!' | '#' | '$' | '&'..=';' | '=' | '?'..='Z' | '_' | 'a'..='z' | '~' => true,
            _ => false,
        };
        if keep {
            result.push(character);
            if character == '%' {
                // The two characters of the escape are kept as they are
                for _ in 0..2 {
                    if let Some((_, next)) = chars.next() {
                        result.push(next);
                    }
                }
            }
            continue;
        }
        let mut buffer = [0; 4];
        for byte in character.encode_utf8(&mut buffer).bytes() {
            result.push_str(&format!("%{byte:02X}"));
        }
    }
    result
}
