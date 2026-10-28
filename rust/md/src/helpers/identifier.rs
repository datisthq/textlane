/// Normalize a label to the identifier definitions match on, as micromark does: whitespace
/// runs collapse to one space, the edges are trimmed and case folds.
pub(crate) fn normalize_identifier(value: &str) -> String {
    let mut collapsed = String::with_capacity(value.len());
    let mut in_whitespace = false;
    for character in value.chars() {
        if matches!(character, '\t' | '\n' | '\r' | ' ') {
            in_whitespace = true;
            continue;
        }
        if in_whitespace && !collapsed.is_empty() {
            collapsed.push(' ');
        }
        in_whitespace = false;
        collapsed.push(character);
    }
    // Lower then upper case, as micromark does, so `ẞ` and `ß` match
    collapsed.to_lowercase().to_uppercase()
}
