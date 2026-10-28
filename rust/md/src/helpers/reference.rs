use crate::helpers::entities::decode_named;

const DECIMAL_SIZE_MAX: usize = 7;
const HEXADECIMAL_SIZE_MAX: usize = 6;
const NAMED_SIZE_MAX: usize = 31;

/// The most characters a reference value may have after its marker (`&`, `#` or `x`).
pub(crate) fn measure_reference(marker: u8) -> usize {
    match marker {
        b'x' => HEXADECIMAL_SIZE_MAX,
        b'#' => DECIMAL_SIZE_MAX,
        _ => NAMED_SIZE_MAX,
    }
}

/// Whether `byte` may appear in a reference value after `marker`.
pub(crate) fn accept_reference_byte(marker: u8, byte: u8) -> bool {
    match marker {
        b'x' => byte.is_ascii_hexdigit(),
        b'#' => byte.is_ascii_digit(),
        _ => byte.is_ascii_alphanumeric(),
    }
}

/// The text a reference value stands for, or nothing for an unknown name.
pub(crate) fn decode_reference(value: &str, marker: u8) -> Option<String> {
    match marker {
        b'#' => Some(decode_numeric(value, 10).to_string()),
        b'x' => Some(decode_numeric(value, 16).to_string()),
        _ => decode_named(value).map(str::to_string),
    }
}

// Controls, lone surrogates and values past Unicode become the replacement character
fn decode_numeric(value: &str, radix: u32) -> char {
    u32::from_str_radix(value, radix)
        .ok()
        .and_then(char::from_u32)
        .filter(|character| {
            !matches!(character,
                '\0'..='\u{08}' | '\u{0B}' | '\u{0E}'..='\u{1F}' | '\u{7F}'..='\u{9F}')
        })
        .unwrap_or(char::REPLACEMENT_CHARACTER)
}
