/// Whether `character` can start a JavaScript identifier, as estree-util-is-identifier-name
/// decides.
pub(crate) fn is_id_start(character: char) -> bool {
    matches!(character, '$' | '_') || unicode_id_start::is_id_start(character)
}

/// Whether `character` can continue a JSX identifier, which may hold `-`.
pub(crate) fn is_id_cont(character: char) -> bool {
    matches!(character, '$' | '_' | '-' | '\u{200C}' | '\u{200D}')
        || unicode_id_start::is_id_continue(character)
}

/// Whether `character` is white space as JavaScript's `\s` matches it.
pub(crate) fn is_js_whitespace(character: char) -> bool {
    matches!(
        character,
        '\t' | '\n' | '\u{B}' | '\u{C}' | '\r' | ' ' | '\u{A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

/// Whether `source` holds only white space and comments, as micromark-util-events-to-acorn
/// checks: block comments go first, then line comments that a line ending closes.
pub(crate) fn is_empty_script(source: &str) -> bool {
    let without_blocks = strip_block_comments(source);
    let mut rest = without_blocks.as_str();
    loop {
        let Some(at) = rest.find("//") else {
            return rest.chars().all(is_js_whitespace);
        };
        if !rest[..at].chars().all(is_js_whitespace) {
            return false;
        }
        let after = &rest[at + 2..];
        match after.find(['\n', '\r']) {
            Some(end) => rest = &after[end + 1..],
            None => return false,
        }
    }
}

fn strip_block_comments(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut rest = source;
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

/// A character as micromark's MDX messages name one: `` `a` (U+0061) ``, or end of file.
pub(crate) fn format_character(character: Option<char>) -> String {
    match character {
        None => "end of file".into(),
        Some(character) => {
            let shown = if character == '`' {
                "`` ` ``".to_string()
            } else {
                format!("`{character}`")
            };
            format!("character {shown} (U+{:04X})", u32::from(character))
        }
    }
}
