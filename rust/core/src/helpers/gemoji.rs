use crate::helpers::gemoji_table::NAMES;

/// The `:name:` shortcodes gemoji knows in `value`, each `(start, end, emoji)`, found as
/// remark-gemoji's `/:(\+1|[-\w]+):/g` finds them; an unknown name is retried from its
/// next character, so its closing colon can open the next shortcode.
pub fn find_gemoji_shortcodes(value: &str) -> Vec<(usize, usize, &'static str)> {
    let bytes = value.as_bytes();
    let mut found = Vec::new();
    let mut search = 0;
    while let Some(offset) = bytes[search..].iter().position(|byte| *byte == b':') {
        let start = search + offset;
        let Some(end) = find_shortcode_end(bytes, start) else {
            search = start + 1;
            continue;
        };
        let name = &value[start + 1..end - 1];
        match NAMES.binary_search_by(|(each, _)| each.cmp(&name)) {
            Ok(index) => {
                found.push((start, end, NAMES[index].1));
                search = end;
            }
            Err(_) => search = start + 1,
        }
    }
    found
}

fn find_shortcode_end(bytes: &[u8], start: usize) -> Option<usize> {
    let rest = &bytes[start + 1..];
    if rest.starts_with(b"+1:") {
        return Some(start + 4);
    }
    let size = rest
        .iter()
        .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        .count();
    (size > 0 && rest.get(size) == Some(&b':')).then_some(start + size + 2)
}

/// A shortcode's name as remark-emoji labels it: each `_`, and each `-` not before `1`,
/// as a space.
pub fn read_gemoji_label(name: &str) -> String {
    let mut label = String::with_capacity(name.len());
    let mut characters = name.chars().peekable();
    while let Some(character) = characters.next() {
        let dash = character == '-' && characters.peek() != Some(&'1');
        label.push(if character == '_' || dash {
            ' '
        } else {
            character
        });
    }
    label
}
