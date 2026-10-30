use crate::helpers::character::CharacterKind;
use crate::helpers::literal::{
    can_start_email, can_start_protocol, can_start_www, classify_unit_at, is_atext,
};

/// Which literal a GFM autolink is.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum LiteralKind {
    Email,
    Www,
    Http,
}

/// What failed scans saw, as micromark-extension-gfm-autolink-literal's port keeps it: a
/// trail check or domain starting inside a failed one fails the same way.
#[derive(Default)]
pub(crate) struct LiteralMemo {
    trail: (usize, usize),
    domain_failed: (usize, usize),
}

/// A GFM literal autolink at `at`: an email, then a `www.` or `http(s)://` literal, tried
/// in micromark's order; its end and kind.
pub(crate) fn scan_literal(
    bytes: &[u8],
    at: usize,
    previous: Option<u8>,
    memo: &mut LiteralMemo,
) -> Option<(usize, LiteralKind)> {
    let byte = *bytes.get(at)?;
    if can_start_email(previous, byte)
        && let Some(end) = scan_email(bytes, at)
    {
        return Some((end, LiteralKind::Email));
    }
    if can_start_www(previous, byte) {
        return scan_www(bytes, at, memo).map(|end| (end, LiteralKind::Www));
    }
    if can_start_protocol(previous, byte) {
        return scan_protocol(bytes, at, memo).map(|end| (end, LiteralKind::Http));
    }
    None
}

fn scan_email(bytes: &[u8], at: usize) -> Option<usize> {
    let mut index = at;
    loop {
        match bytes.get(index) {
            Some(b'@') => break,
            Some(&byte) if is_atext(byte) => index += 1,
            _ => return None,
        }
    }
    index += 1;
    let (mut seen, mut dot) = (false, false);
    loop {
        match bytes.get(index) {
            Some(b'.') => {
                // A dot before something other than an alphanumeric trails
                if !bytes.get(index + 1).is_some_and(u8::is_ascii_alphanumeric) {
                    break;
                }
                dot = true;
                index += 1;
            }
            Some(&byte)
                if byte == b'-' || byte == b'_' || byte.is_ascii_alphanumeric() =>
            {
                seen = true;
                index += 1;
            }
            _ => break,
        }
    }
    let last_alpha = index > 0 && bytes[index - 1].is_ascii_alphabetic();
    (seen && dot && last_alpha).then_some(index)
}

fn scan_www(bytes: &[u8], at: usize, memo: &mut LiteralMemo) -> Option<usize> {
    let prefix = bytes.get(at..at + 4)?;
    if !prefix[..3].eq_ignore_ascii_case(b"www")
        || prefix[3] != b'.'
        || at + 4 >= bytes.len()
    {
        return None;
    }
    let domain_end = scan_domain(bytes, at, memo)?;
    Some(scan_path(bytes, domain_end, memo))
}

fn scan_protocol(bytes: &[u8], at: usize, memo: &mut LiteralMemo) -> Option<usize> {
    let mut index = at + 1;
    while index < bytes.len() && bytes[index].is_ascii_alphabetic() && index - at < 5 {
        index += 1;
    }
    if bytes.get(index) != Some(&b':') {
        return None;
    }
    let name = &bytes[at..index];
    if !(name.eq_ignore_ascii_case(b"http") || name.eq_ignore_ascii_case(b"https")) {
        return None;
    }
    if bytes.get(index + 1..index + 3) != Some(b"//") {
        return None;
    }
    let start = index + 3;
    match bytes.get(start) {
        None => return None,
        Some(&byte) if byte.is_ascii_control() => return None,
        Some(&byte)
            if classify_unit_at(bytes, start, Some(byte)) != CharacterKind::Other =>
        {
            return None;
        }
        _ => {}
    }
    let domain_end = scan_domain(bytes, start, memo)?;
    Some(scan_path(bytes, domain_end, memo))
}

fn scan_domain(bytes: &[u8], at: usize, memo: &mut LiteralMemo) -> Option<usize> {
    let (failed_start, failed_dot) = memo.domain_failed;
    if failed_start <= at && at < failed_dot {
        return None;
    }
    let mut index = at;
    let (mut seen, mut marker, mut marker_before) = (false, 0u8, 0u8);
    let (mut last_dot, mut second_last) = (0, 0);
    loop {
        let byte = bytes.get(index).copied();
        match byte {
            Some(b'.' | b'_') => {
                if scan_trail(bytes, index, memo) {
                    break;
                }
                if byte == Some(b'_') {
                    marker = b'_';
                } else {
                    marker_before = marker;
                    marker = 0;
                    second_last = last_dot;
                    last_dot = index;
                }
                index += 1;
            }
            Some(0x80..=0xBF | b'-') => index += 1,
            _ if classify_unit_at(bytes, index, byte) != CharacterKind::Other => break,
            _ => {
                seen = true;
                index += 1;
            }
        }
    }
    if marker_before != b'_' && marker != b'_' && seen {
        return Some(index);
    }
    memo.domain_failed = (at, second_last);
    None
}

fn scan_path(bytes: &[u8], at: usize, memo: &mut LiteralMemo) -> usize {
    let mut index = at;
    let (mut open, mut closed) = (0usize, 0usize);
    loop {
        let byte = bytes.get(index).copied();
        match byte {
            Some(b'(') => {
                open += 1;
                index += 1;
            }
            // A closing paren with an opening one left for it is never trailing
            Some(b')') if closed < open => {
                closed += 1;
                index += 1;
            }
            Some(
                b'!' | b'"' | b'&' | b'\'' | b')' | b'*' | b',' | b'.' | b':' | b';'
                | b'<' | b'?' | b']' | b'_' | b'~',
            ) => {
                if scan_trail(bytes, index, memo) {
                    return index;
                }
                if byte == Some(b')') {
                    closed += 1;
                }
                index += 1;
            }
            Some(0x80..=0xBF) => index += 1,
            _ if classify_unit_at(bytes, index, byte) == CharacterKind::Whitespace => {
                return index;
            }
            _ => index += 1,
        }
    }
}

// Whether trailing punctuation from `at` runs to the end of the literal
fn scan_trail(bytes: &[u8], at: usize, memo: &mut LiteralMemo) -> bool {
    let (start, failed) = memo.trail;
    if start <= at && at < failed {
        return false;
    }
    memo.trail = (at, 0);
    let mut index = at;
    loop {
        let byte = bytes.get(index).copied();
        match byte {
            Some(
                b'!' | b'"' | b'\'' | b')' | b'*' | b',' | b'.' | b':' | b';' | b'?'
                | b'_' | b'~',
            ) => {
                index += 1;
            }
            // `&`, letters and `;` look like a character reference and trail as a whole
            Some(b'&') => {
                index += 1;
                if !bytes.get(index).is_some_and(u8::is_ascii_alphabetic) {
                    memo.trail.1 = index;
                    return false;
                }
                while bytes.get(index).is_some_and(u8::is_ascii_alphabetic) {
                    index += 1;
                }
                if bytes.get(index) != Some(&b';') {
                    memo.trail.1 = index;
                    return false;
                }
                index += 1;
            }
            // A `]` trails unless a resource or reference could follow it
            Some(b']') => {
                index += 1;
                let next = bytes.get(index).copied();
                if matches!(next, Some(b'(' | b'['))
                    || classify_unit_at(bytes, index, next) == CharacterKind::Whitespace
                {
                    return true;
                }
            }
            Some(b'<') => return true,
            _ if classify_unit_at(bytes, index, byte) == CharacterKind::Whitespace => {
                return true;
            }
            _ => {
                memo.trail.1 = index;
                return false;
            }
        }
    }
}
