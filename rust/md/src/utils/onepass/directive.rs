use crate::helpers::attribute::decode_attribute;
use crate::helpers::character::{
    CharacterKind, classify_character, classify_character_unit, find_character_after,
};

/// A parsed directive head: where its name, label content and attributes lie.
pub(crate) struct DirectiveHead {
    pub(crate) name: (usize, usize),
    /// The label's content, inside its brackets, and the label with them.
    pub(crate) label: Option<((usize, usize), (usize, usize))>,
    pub(crate) attributes: Vec<(String, String)>,
    pub(crate) end: usize,
}

// As micromark classifies a code: the end and line endings are whitespace; a tab and the
// inside of a multibyte character are neither
fn classify(bytes: &[u8], at: usize, end: usize) -> CharacterKind {
    match bytes.get(at).filter(|_| at < end) {
        None => CharacterKind::Whitespace,
        Some(b'\t') => CharacterKind::Other,
        Some(&byte) if byte.is_ascii() => classify_character(Some(char::from(byte))),
        Some(0x80..=0xBF) => CharacterKind::Other,
        Some(_) => classify_character_unit(find_character_after(bytes, at)),
    }
}

/// A directive's name at `at`: its end.
pub(crate) fn scan_name(bytes: &[u8], at: usize, end: usize) -> Option<usize> {
    if classify(bytes, at, end) != CharacterKind::Other {
        return None;
    }
    let mut index = at + 1;
    while index < end
        && (classify(bytes, index, end) == CharacterKind::Other
            || matches!(bytes[index], b'-' | b'_'))
    {
        index += 1;
    }
    (!matches!(bytes[index - 1], b'-' | b'_')).then_some(index)
}

/// A directive's label at `at`, `[…]`: its content's range and the index after it; brackets
/// balance at most 32 deep, and only a text directive's label spans lines.
pub(crate) fn scan_label(
    bytes: &[u8],
    at: usize,
    end: usize,
    eol: bool,
) -> Option<((usize, usize), usize)> {
    if bytes.get(at) != Some(&b'[') {
        return None;
    }
    let mut index = at + 1;
    let mut balance = 0;
    loop {
        if index >= end {
            return None;
        }
        match bytes[index] {
            b'[' => {
                balance += 1;
                if balance > 32 {
                    return None;
                }
                index += 1;
            }
            b']' if balance == 0 => return Some(((at + 1, index), index + 1)),
            b']' => {
                balance -= 1;
                index += 1;
            }
            b'\n' | b'\r' if !eol => return None,
            b'\\' if matches!(bytes.get(index + 1), Some(b'[' | b'\\' | b']')) => {
                index += 2
            }
            _ => index += 1,
        }
    }
}

fn is_space(byte: u8) -> bool {
    byte == b' ' || byte == b'\t'
}

fn is_eol(byte: u8) -> bool {
    byte == b'\n' || byte == b'\r'
}

// Spaces and tabs, and with `eol` one line ending among them: the index after them
fn skip_whitespace(bytes: &[u8], at: usize, end: usize, eol: bool) -> usize {
    let mut index = at;
    let mut seen_eol = false;
    while index < end {
        match bytes[index] {
            byte if is_space(byte) => index += 1,
            byte if eol && is_eol(byte) && !seen_eol => {
                seen_eol = true;
                index += if byte == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
                    2
                } else {
                    1
                };
            }
            _ => break,
        }
    }
    index
}

fn is_name_end(bytes: &[u8], at: usize, end: usize, allowed: &[u8]) -> bool {
    match classify(bytes, at, end) {
        CharacterKind::Whitespace => true,
        CharacterKind::Punctuation => !allowed.contains(&bytes[at]),
        CharacterKind::Other => false,
    }
}

/// A directive's attributes at `at`, `{…}`: each key and decoded value in order, classes
/// joined and later keys replacing earlier ones, and the index after them.
pub(crate) fn scan_attributes(
    bytes: &[u8],
    text: &str,
    at: usize,
    end: usize,
    eol: bool,
) -> Option<(Vec<(String, String)>, usize)> {
    if bytes.get(at) != Some(&b'{') {
        return None;
    }
    let mut list: Vec<(String, String)> = vec![];
    let mut index = at + 1;
    loop {
        if index >= end {
            return None;
        }
        let byte = bytes[index];
        if byte == b'#' || byte == b'.' {
            let start = index + 1;
            match bytes.get(start).filter(|_| start < end) {
                None => return None,
                Some(
                    b'"' | b'#' | b'\'' | b'.' | b'<' | b'=' | b'>' | b'`' | b'}' | b'\t'
                    | b'\n' | b'\r' | b' ',
                ) => {
                    return None;
                }
                _ => {}
            }
            let mut stop = start + 1;
            loop {
                match bytes.get(stop).filter(|_| stop < end) {
                    None | Some(b'"' | b'\'' | b'<' | b'=' | b'>' | b'`') => return None,
                    Some(b'#' | b'.' | b'}' | b'\t' | b'\n' | b'\r' | b' ') => break,
                    Some(_) => stop += 1,
                }
            }
            let key = if byte == b'.' { "class" } else { "id" };
            list.push((key.to_string(), decode_attribute(&text[start..stop])));
            index = stop;
            continue;
        }
        if is_space(byte) || (eol && is_eol(byte)) {
            let next = skip_whitespace(bytes, index, end, eol);
            if next == index {
                return None;
            }
            index = next;
            continue;
        }
        if is_eol(byte) {
            return None;
        }
        if is_name_end(bytes, index, end, b"-_") {
            return (byte == b'}').then(|| (merge(list), index + 1));
        }
        let name_start = index;
        index += 1;
        while index < end && !is_name_end(bytes, index, end, b"-.:_") {
            index += 1;
        }
        let key = text[name_start..index].to_string();
        let after_name = skip_whitespace(bytes, index, end, eol);
        if bytes.get(after_name).filter(|_| after_name < end) != Some(&b'=') {
            list.push((key, String::new()));
            index = after_name;
            continue;
        }
        let mut value_at = after_name + 1;
        loop {
            match bytes.get(value_at).filter(|_| value_at < end) {
                None | Some(b'<' | b'=' | b'>' | b'`' | b'}') => return None,
                Some(&byte) if is_eol(byte) && !eol => return None,
                Some(&byte) if is_space(byte) || is_eol(byte) => {
                    let next = skip_whitespace(bytes, value_at, end, eol);
                    if next == value_at {
                        return None;
                    }
                    value_at = next;
                }
                _ => break,
            }
        }
        let quote = bytes[value_at];
        if quote == b'"' || quote == b'\'' {
            let start = value_at + 1;
            let mut stop = start;
            loop {
                match bytes.get(stop).filter(|_| stop < end) {
                    None => return None,
                    Some(&byte) if byte == quote => break,
                    Some(&byte) if is_eol(byte) && !eol => return None,
                    Some(_) => stop += 1,
                }
            }
            list.push((key, decode_attribute(&text[start..stop])));
            index = stop + 1;
            match bytes.get(index).filter(|_| index < end) {
                Some(b'}' | b'\t' | b'\n' | b'\r' | b' ') => {}
                _ => {
                    return (bytes.get(index) == Some(&b'}'))
                        .then(|| (merge(list), index + 1));
                }
            }
        } else {
            let start = value_at;
            let mut stop = start + 1;
            loop {
                match bytes.get(stop).filter(|_| stop < end) {
                    None | Some(b'"' | b'\'' | b'<' | b'=' | b'>' | b'`') => return None,
                    Some(b'}' | b'\t' | b'\n' | b'\r' | b' ') => break,
                    Some(_) => stop += 1,
                }
            }
            list.push((key, decode_attribute(&text[start..stop])));
            index = stop;
        }
    }
}

// Classes join; any other repeated key keeps its first place and its last value
fn merge(list: Vec<(String, String)>) -> Vec<(String, String)> {
    let mut merged: Vec<(String, String)> = Vec::with_capacity(list.len());
    for (key, value) in list {
        match merged.iter_mut().find(|(existing, _)| *existing == key) {
            Some((_, existing)) if key == "class" => {
                existing.push(' ');
                existing.push_str(&value);
            }
            Some((_, existing)) => *existing = value,
            None => merged.push((key, value)),
        }
    }
    merged
}

/// A directive's name, then an optional label and attributes, from `at`.
pub(crate) fn scan_head(
    bytes: &[u8],
    text: &str,
    at: usize,
    end: usize,
    eol: bool,
    text_directive: bool,
) -> Option<DirectiveHead> {
    let name_end = scan_name(bytes, at, end)?;
    if text_directive && bytes.get(name_end).filter(|_| name_end < end) == Some(&b':') {
        return None;
    }
    let mut index = name_end;
    let mut label = None;
    if let Some((content, after)) = scan_label(bytes, index, end, eol) {
        label = Some((content, (index, after)));
        index = after;
    }
    let mut attributes = vec![];
    if let Some((list, after)) = scan_attributes(bytes, text, index, end, eol) {
        attributes = list;
        index = after;
    }
    Some(DirectiveHead {
        name: (at, name_end),
        label,
        attributes,
        end: index,
    })
}
