use std::borrow::Cow;

/// As the trim-lines package does: lines lose the spaces and tabs around their line
/// endings, the first keeping its start and the last its end.
pub(crate) fn trim_lines(value: &str) -> Cow<'_, str> {
    let bytes = value.as_bytes();
    let blank = |at: usize| matches!(bytes.get(at), Some(b' ' | b'\t'));
    if !memchr::memchr2_iter(b'\n', b'\r', bytes)
        .any(|index| (index > 0 && blank(index - 1)) || blank(index + 1))
    {
        return Cow::Borrowed(value);
    }
    let mut result = String::with_capacity(value.len());
    let mut last = 0;
    for index in memchr::memchr2_iter(b'\n', b'\r', bytes) {
        // The `\n` of a `\r\n` was taken with its `\r`
        if index < last {
            continue;
        }
        let ending = match bytes[index] {
            b'\r' if bytes.get(index + 1) == Some(&b'\n') => 2,
            _ => 1,
        };
        result.push_str(trim_line(&value[last..index], last > 0, true));
        result.push_str(&value[index..index + ending]);
        last = index + ending;
    }
    result.push_str(trim_line(&value[last..], last > 0, false));
    Cow::Owned(result)
}

fn trim_line(line: &str, start: bool, end: bool) -> &str {
    let is_blank = |c: char| c == ' ' || c == '\t';
    let line = if start {
        line.trim_start_matches(is_blank)
    } else {
        line
    };
    if end {
        line.trim_end_matches(is_blank)
    } else {
        line
    }
}
