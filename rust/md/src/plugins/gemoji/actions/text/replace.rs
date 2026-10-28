use crate::plugins::gemoji::names::NAMES;

/// Replace each `:name:` gemoji knows with its emoji, as remark-gemoji's
/// `/:(\+1|[-\w]+):/g` finds them; `None` when nothing is replaced.
pub(crate) fn replace_shortcodes(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut result = String::new();
    let mut copied = 0;
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
                result.push_str(&value[copied..start]);
                result.push_str(NAMES[index].1);
                copied = end;
                search = end;
            }
            // An unknown name is retried from its next character, so its closing colon can
            // open the next shortcode
            Err(_) => search = start + 1,
        }
    }
    if copied == 0 {
        return None;
    }
    result.push_str(&value[copied..]);
    Some(result)
}

// The end of the shortcode whose opening colon is at `start`, past its closing colon
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
