use std::fmt::Write;

/// `JSON.stringify` of a string.
pub(crate) fn json_string(value: &str) -> String {
    let mut result = String::with_capacity(value.len() + 2);
    result.push('"');
    for character in value.chars() {
        match character {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\u{8}' => result.push_str("\\b"),
            '\u{c}' => result.push_str("\\f"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\0'..='\u{1f}' => {
                let _ = write!(result, "\\u{:04x}", u32::from(character));
            }
            _ => result.push(character),
        }
    }
    result.push('"');
    result
}

/// JS's `String(number)` for the numbers a tree holds.
pub(crate) fn format_number(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    if value == 0.0 {
        return "0".to_string();
    }
    if value.fract() == 0.0 && value.abs() < 1e21 {
        return format!("{value:.0}");
    }
    let magnitude = value.abs();
    if (1e-7..1e21).contains(&magnitude) {
        return format!("{value}");
    }
    // JS writes the shortest digits with an `e+`/`e-` exponent outside that range
    let text = format!("{value:e}");
    match text.split_once('e') {
        Some((mantissa, exponent)) if !exponent.starts_with('-') => {
            format!("{mantissa}e+{exponent}")
        }
        _ => text,
    }
}
