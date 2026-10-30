use crate::helpers::html::{BLOCK_NAMES, RAW_NAMES};
use crate::helpers::reference::{accept_reference_byte, measure_reference};

pub(crate) fn is_space_or_tab(byte: u8) -> bool {
    byte == b' ' || byte == b'\t'
}

pub(crate) fn is_line_ending(byte: u8) -> bool {
    byte == b'\n' || byte == b'\r'
}

/// Whether only spaces and tabs remain from `at` to `end`.
pub(crate) fn is_blank_from(bytes: &[u8], at: usize, end: usize) -> bool {
    bytes[at..end].iter().all(|byte| is_space_or_tab(*byte))
}

/// A thematic break at `at`: three or more of one marker, spaces and tabs between.
pub(crate) fn scan_thematic_break(bytes: &[u8], at: usize, end: usize) -> bool {
    let Some(&marker @ (b'*' | b'-' | b'_')) = bytes.get(at) else {
        return false;
    };
    let mut count = 0;
    for &byte in &bytes[at..end] {
        if byte == marker {
            count += 1;
        } else if !is_space_or_tab(byte) {
            return false;
        }
    }
    count >= 3
}

/// An ATX opening at `at`: its depth and where the content after the sequence starts.
pub(crate) fn scan_atx(bytes: &[u8], at: usize, end: usize) -> Option<(u8, usize)> {
    let mut index = at;
    while index < end && bytes[index] == b'#' {
        index += 1;
    }
    let depth = index - at;
    if depth == 0 || depth > 6 || (index < end && !is_space_or_tab(bytes[index])) {
        return None;
    }
    Some((depth as u8, index))
}

/// The content of an ATX heading between `from` and `end`: trimmed, closing sequence off.
pub(crate) fn trim_atx_content(bytes: &[u8], from: usize, end: usize) -> (usize, usize) {
    let mut start = from;
    while start < end && is_space_or_tab(bytes[start]) {
        start += 1;
    }
    let mut stop = end;
    while stop > start && is_space_or_tab(bytes[stop - 1]) {
        stop -= 1;
    }
    let mut hashes = stop;
    while hashes > start && bytes[hashes - 1] == b'#' {
        hashes -= 1;
    }
    if hashes < stop && (hashes == start || is_space_or_tab(bytes[hashes - 1])) {
        stop = hashes;
        while stop > start && is_space_or_tab(bytes[stop - 1]) {
            stop -= 1;
        }
    }
    (start, stop)
}

/// A code fence opening at `at`: its marker and size, or nothing when the info string of a
/// backtick fence holds a backtick.
pub(crate) fn scan_fence_open(
    bytes: &[u8],
    at: usize,
    end: usize,
    math: bool,
) -> Option<(u8, usize)> {
    let marker = *bytes.get(at)?;
    if marker != b'`' && marker != b'~' && !(math && marker == b'$') {
        return None;
    }
    let mut index = at;
    while index < end && bytes[index] == marker {
        index += 1;
    }
    let size = index - at;
    let least = if marker == b'$' { 2 } else { 3 };
    // Backticks and dollars may not appear in their own fence's info
    if size < least || (marker != b'~' && bytes[index..end].contains(&marker)) {
        return None;
    }
    Some((marker, size))
}

/// Whether the line from `at` closes a fence of `marker` and `size`.
pub(crate) fn scan_fence_close(
    bytes: &[u8],
    at: usize,
    end: usize,
    marker: u8,
    size: usize,
) -> bool {
    let mut index = at;
    while index < end && bytes[index] == marker {
        index += 1;
    }
    index - at >= size && is_blank_from(bytes, index, end)
}

/// A setext underline at `at`: its depth.
pub(crate) fn scan_setext(bytes: &[u8], at: usize, end: usize) -> Option<u8> {
    let marker = *bytes.get(at)?;
    if marker != b'=' && marker != b'-' {
        return None;
    }
    let mut index = at;
    while index < end && bytes[index] == marker {
        index += 1;
    }
    is_blank_from(bytes, index, end).then_some(if marker == b'=' { 1 } else { 2 })
}

/// A list marker at `at`: ordered, the bullet or delimiter, the start number and its size.
pub(crate) fn scan_list_marker(
    bytes: &[u8],
    at: usize,
    end: usize,
) -> Option<(bool, u8, u32, usize)> {
    let byte = *bytes.get(at).filter(|_| at < end)?;
    let (ordered, marker, start, size) = match byte {
        b'*' | b'+' | b'-' => (false, byte, 0, 1),
        b'0'..=b'9' => {
            let mut index = at;
            while index < end && index - at < 10 && bytes[index].is_ascii_digit() {
                index += 1;
            }
            if index - at > 9 || index >= end || !matches!(bytes[index], b'.' | b')') {
                return None;
            }
            let start = std::str::from_utf8(&bytes[at..index]).ok()?.parse().ok()?;
            (true, bytes[index], start, index + 1 - at)
        }
        _ => return None,
    };
    let after = at + size;
    if after < end && !is_space_or_tab(bytes[after]) {
        return None;
    }
    Some((ordered, marker, start, size))
}

/// The kind of HTML block starting at `at`, 1 to 7, as CommonMark numbers them.
pub(crate) fn scan_html_start(
    bytes: &[u8],
    at: usize,
    end: usize,
    interrupt: bool,
) -> Option<u8> {
    let rest = &bytes[at..end];
    if rest.first() != Some(&b'<') {
        return None;
    }
    if rest.starts_with(b"<!--") {
        return Some(2);
    }
    if rest.starts_with(b"<?") {
        return Some(3);
    }
    if rest.starts_with(b"<![CDATA[") {
        return Some(5);
    }
    if rest.len() > 2 && rest[1] == b'!' && rest[2].is_ascii_alphabetic() {
        return Some(4);
    }
    let closing = rest.get(1) == Some(&b'/');
    let name_start = if closing { 2 } else { 1 };
    let mut name_end = name_start;
    while name_end < rest.len() && rest[name_end].is_ascii_alphanumeric() {
        name_end += 1;
    }
    let name = &rest[name_start..name_end];
    let after = rest.get(name_end).copied();
    let ends_name = |after: Option<u8>| {
        matches!(after, None | Some(b' ' | b'\t' | b'>'))
            || (after == Some(b'/') && rest.get(name_end + 1) == Some(&b'>'))
    };
    if !closing
        && matches!(after, None | Some(b' ' | b'\t' | b'>'))
        && RAW_NAMES
            .iter()
            .any(|raw| raw.as_bytes().eq_ignore_ascii_case(name))
    {
        return Some(1);
    }
    if ends_name(after)
        && BLOCK_NAMES
            .iter()
            .any(|block| block.as_bytes().eq_ignore_ascii_case(name))
    {
        return Some(6);
    }
    if interrupt {
        return None;
    }
    let tag_end = if closing {
        scan_closing_tag(rest, 0, false)
    } else {
        scan_open_tag(rest, 0, false)
    }?;
    is_blank_from(rest, tag_end, rest.len()).then_some(7)
}

/// Whether a line of an HTML block of `kind` holds its end.
pub(crate) fn scan_html_end(line: &[u8], kind: u8) -> bool {
    match kind {
        1 => RAW_NAMES.iter().any(|name| {
            line.windows(name.len() + 3).any(|window| {
                window.starts_with(b"</")
                    && window[2..2 + name.len()].eq_ignore_ascii_case(name.as_bytes())
                    && window[2 + name.len()] == b'>'
            })
        }),
        2 => contains(line, b"-->"),
        3 => contains(line, b"?>"),
        4 => line.contains(&b'>'),
        5 => cdata_ends(line),
        _ => false,
    }
}

// micromark's states: a `]` after `]]` counts as the first of a new pair, so `]]]>` does
// not end the section
fn cdata_ends(line: &[u8]) -> bool {
    let mut brackets = 0;
    for &byte in line {
        brackets = match (brackets, byte) {
            (2, b'>') => return true,
            (2, b']') => 1,
            (_, b']') => brackets + 1,
            _ => 0,
        };
    }
    false
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

// Whitespace inside a tag; line endings only where `eol` allows them
fn skip_tag_space(bytes: &[u8], mut index: usize, eol: bool) -> usize {
    while index < bytes.len()
        && (is_space_or_tab(bytes[index]) || (eol && is_line_ending(bytes[index])))
    {
        index += 1;
    }
    index
}

/// An open tag at `at`, `<name attributes>`: the index after it.
pub(crate) fn scan_open_tag(bytes: &[u8], at: usize, eol: bool) -> Option<usize> {
    scan_open_tag_with(bytes, at, eol, None)
}

fn scan_open_tag_with(
    bytes: &[u8],
    at: usize,
    eol: bool,
    mut memo: Option<&mut HtmlMemo>,
) -> Option<usize> {
    let mut index = at + 1;
    if !bytes.get(index)?.is_ascii_alphabetic() {
        return None;
    }
    while index < bytes.len()
        && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'-')
    {
        index += 1;
    }
    loop {
        let after_space = skip_tag_space(bytes, index, eol);
        match bytes.get(after_space) {
            Some(b'>') => return Some(after_space + 1),
            Some(b'/') => {
                return (bytes.get(after_space + 1) == Some(&b'>'))
                    .then_some(after_space + 2);
            }
            Some(&byte)
                if after_space > index
                    && (byte.is_ascii_alphabetic() || byte == b'_' || byte == b':') =>
            {
                index = after_space + 1;
                while index < bytes.len()
                    && (bytes[index].is_ascii_alphanumeric()
                        || matches!(bytes[index], b'_' | b'.' | b':' | b'-'))
                {
                    index += 1;
                }
                let before_value = skip_tag_space(bytes, index, eol);
                if bytes.get(before_value) == Some(&b'=') {
                    let value = skip_tag_space(bytes, before_value + 1, eol);
                    index = scan_attribute_value(bytes, value, eol, memo.as_deref_mut())?;
                }
            }
            _ => return None,
        }
    }
}

fn scan_attribute_value(
    bytes: &[u8],
    at: usize,
    eol: bool,
    memo: Option<&mut HtmlMemo>,
) -> Option<usize> {
    match *bytes.get(at)? {
        quote @ (b'"' | b'\'') if eol => {
            let slot = if quote == b'"' { 4 } else { 5 };
            match memo {
                Some(memo) => memo.search(slot, bytes, at + 1, &[quote]),
                None => find(bytes, at + 1, &[quote]),
            }
            .map(|end| end + 1)
        }
        quote @ (b'"' | b'\'') => {
            let mut index = at + 1;
            while index < bytes.len() && bytes[index] != quote {
                if !eol && is_line_ending(bytes[index]) {
                    return None;
                }
                index += 1;
            }
            (index < bytes.len()).then_some(index + 1)
        }
        _ => {
            let mut index = at;
            while index < bytes.len()
                && !matches!(
                    bytes[index],
                    b' ' | b'\t'
                        | b'\n'
                        | b'\r'
                        | b'"'
                        | b'\''
                        | b'='
                        | b'<'
                        | b'>'
                        | b'`'
                )
            {
                index += 1;
            }
            (index > at).then_some(index)
        }
    }
}

/// A closing tag at `at`, `</name>`: the index after it.
pub(crate) fn scan_closing_tag(bytes: &[u8], at: usize, eol: bool) -> Option<usize> {
    let mut index = at + 2;
    if bytes.get(at + 1) != Some(&b'/') || !bytes.get(index)?.is_ascii_alphabetic() {
        return None;
    }
    while index < bytes.len()
        && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'-')
    {
        index += 1;
    }
    let index = skip_tag_space(bytes, index, eol);
    (bytes.get(index) == Some(&b'>')).then_some(index + 1)
}

/// Where searches for the end of inline HTML failed: a later search from past there
/// fails too, since it reads to the end of the same text.
#[derive(Default)]
pub(crate) struct HtmlMemo {
    failed: [usize; 6],
    set: [bool; 6],
}

impl HtmlMemo {
    fn search(
        &mut self,
        slot: usize,
        bytes: &[u8],
        from: usize,
        needle: &[u8],
    ) -> Option<usize> {
        if self.set[slot] && from >= self.failed[slot] {
            return None;
        }
        let found = find(bytes, from, needle);
        if found.is_none() && (!self.set[slot] || from < self.failed[slot]) {
            self.set[slot] = true;
            self.failed[slot] = from;
        }
        found
    }
}

/// Inline HTML at `at`: the index after it.
pub(crate) fn scan_html_text(
    bytes: &[u8],
    at: usize,
    memo: &mut HtmlMemo,
) -> Option<usize> {
    let rest = &bytes[at..];
    if rest.starts_with(b"<!--") {
        if rest.get(4) == Some(&b'>') {
            return Some(at + 5);
        }
        if rest[4..].starts_with(b"->") {
            return Some(at + 6);
        }
        return memo.search(0, bytes, at + 4, b"-->").map(|end| end + 3);
    }
    if rest.starts_with(b"<?") {
        return memo.search(1, bytes, at + 2, b"?>").map(|end| end + 2);
    }
    if rest.starts_with(b"<![CDATA[") {
        return memo.search(2, bytes, at + 9, b"]]>").map(|end| end + 3);
    }
    if rest.starts_with(b"<!") {
        if !rest.get(2)?.is_ascii_alphabetic() {
            return None;
        }
        return memo.search(3, bytes, at + 3, b">").map(|end| end + 1);
    }
    if rest.get(1) == Some(&b'/') {
        return scan_closing_tag(bytes, at, true);
    }
    scan_open_tag_with(bytes, at, true, Some(memo))
}

fn find(haystack: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    haystack
        .get(from..)?
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|found| from + found)
}

/// An autolink at `at`: the index after its `>` and whether it is an email.
pub(crate) fn scan_autolink(bytes: &[u8], at: usize) -> Option<(usize, bool)> {
    let mut index = at + 1;
    if bytes.get(index)?.is_ascii_alphabetic() {
        let scheme_start = index;
        while index < bytes.len()
            && index - scheme_start < 33
            && (bytes[index].is_ascii_alphanumeric()
                || matches!(bytes[index], b'+' | b'.' | b'-'))
        {
            index += 1;
        }
        let size = index - scheme_start;
        if (2..=32).contains(&size) && bytes.get(index) == Some(&b':') {
            index += 1;
            while index < bytes.len() {
                match bytes[index] {
                    b'>' => return Some((index + 1, false)),
                    byte if byte <= b' ' || byte == b'<' || byte == 0x7F => break,
                    _ => index += 1,
                }
            }
        }
    }
    scan_email(bytes, at + 1).map(|end| (end, true))
}

fn scan_email(bytes: &[u8], at: usize) -> Option<usize> {
    let mut index = at;
    while index < bytes.len()
        && (bytes[index].is_ascii_alphanumeric()
            || b".#$%&'*+/=?^_`{|}~-".contains(&bytes[index]))
    {
        index += 1;
    }
    if index == at || bytes.get(index) != Some(&b'@') {
        return None;
    }
    index += 1;
    loop {
        let label = index;
        while index < bytes.len()
            && index - label < 64
            && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'-')
        {
            index += 1;
        }
        let size = index - label;
        if size == 0 || size > 63 || bytes[label] == b'-' || bytes[index - 1] == b'-' {
            return None;
        }
        match bytes.get(index) {
            Some(b'.') => index += 1,
            Some(b'>') => return Some(index + 1),
            _ => return None,
        }
    }
}

/// A character reference at `at`: the index after its `;`.
pub(crate) fn scan_entity(bytes: &[u8], at: usize) -> Option<usize> {
    let (marker, start) = match (bytes.get(at + 1), bytes.get(at + 2)) {
        (Some(b'#'), Some(b'x' | b'X')) => (b'x', at + 3),
        (Some(b'#'), _) => (b'#', at + 2),
        _ => (b'&', at + 1),
    };
    let mut index = start;
    while index < bytes.len()
        && index - start < measure_reference(marker)
        && accept_reference_byte(marker, bytes[index])
    {
        index += 1;
    }
    (index > start && bytes.get(index) == Some(&b';')).then_some(index + 1)
}

/// A link label at `at`, `[…]`: the index after its `]`; at most 999 characters, no
/// unescaped brackets, and something besides whitespace.
pub(crate) fn scan_label(bytes: &[u8], at: usize) -> Option<usize> {
    if bytes.get(at) != Some(&b'[') {
        return None;
    }
    let mut index = at + 1;
    let mut content = false;
    while index < bytes.len() {
        if index - at > 1000 {
            return None;
        }
        match bytes[index] {
            b'[' => return None,
            b']' => return content.then_some(index + 1),
            b'\\'
                if bytes
                    .get(index + 1)
                    .is_some_and(|byte| matches!(byte, b'[' | b']' | b'\\')) =>
            {
                content = true;
                index += 2;
            }
            byte => {
                content |= !matches!(byte, b' ' | b'\t' | b'\n' | b'\r');
                index += 1;
            }
        }
    }
    None
}

/// A link destination at `at`: where its raw value lies and the index after it.
pub(crate) fn scan_destination(bytes: &[u8], at: usize) -> Option<(usize, usize, usize)> {
    if bytes.get(at) == Some(&b'<') {
        let mut index = at + 1;
        while index < bytes.len() {
            match bytes[index] {
                b'>' => return Some((at + 1, index, index + 1)),
                b'<' | b'\n' | b'\r' => return None,
                b'\\' if bytes.get(index + 1).is_some_and(u8::is_ascii_punctuation) => {
                    index += 2
                }
                _ => index += 1,
            }
        }
        return None;
    }
    let mut index = at;
    let mut depth = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' if bytes.get(index + 1).is_some_and(u8::is_ascii_punctuation) => {
                index += 2
            }
            b'(' => {
                depth += 1;
                if depth > 32 {
                    return None;
                }
                index += 1;
            }
            b')' if depth == 0 => break,
            b')' => {
                depth -= 1;
                index += 1;
            }
            // NUL reads as U+FFFD, which a destination may hold
            byte if (byte <= b' ' && byte != 0) || byte == 0x7F => break,
            _ => index += 1,
        }
    }
    (index > at && depth == 0).then_some((at, index, index))
}

/// A link title at `at`: where its raw value lies and the index after it; no blank line.
pub(crate) fn scan_title(bytes: &[u8], at: usize) -> Option<(usize, usize, usize)> {
    let close = match *bytes.get(at)? {
        b'"' => b'"',
        b'\'' => b'\'',
        b'(' => b')',
        _ => return None,
    };
    let mut index = at + 1;
    let mut line_blank = false;
    while index < bytes.len() {
        match bytes[index] {
            byte if byte == close => return Some((at + 1, index, index + 1)),
            b'(' if close == b')' => return None,
            b'\\' if bytes.get(index + 1).is_some_and(u8::is_ascii_punctuation) => {
                line_blank = false;
                index += 2;
            }
            b'\n' | b'\r' => {
                if line_blank {
                    return None;
                }
                line_blank = true;
                index += if bytes[index] == b'\r' && bytes.get(index + 1) == Some(&b'\n')
                {
                    2
                } else {
                    1
                };
            }
            b' ' | b'\t' => index += 1,
            _ => {
                line_blank = false;
                index += 1;
            }
        }
    }
    None
}

/// Spaces, tabs and at most one line ending from `at`: the index after them, and whether a
/// line ending was among them.
pub(crate) fn skip_space_eol(bytes: &[u8], at: usize) -> (usize, bool) {
    let mut index = at;
    let mut eol = false;
    while index < bytes.len() {
        match bytes[index] {
            b' ' | b'\t' => index += 1,
            b'\n' | b'\r' if !eol => {
                eol = true;
                index += if bytes[index] == b'\r' && bytes.get(index + 1) == Some(&b'\n')
                {
                    2
                } else {
                    1
                };
            }
            _ => break,
        }
    }
    (index, eol)
}

/// The label of a GFM footnote after `[^`, through `end`: the index of its `]`.
pub(crate) fn scan_footnote_label(
    bytes: &[u8],
    from: usize,
    end: usize,
) -> Option<usize> {
    let mut index = from;
    let mut size = 0;
    loop {
        if index >= end {
            return None;
        }
        match bytes[index] {
            b'\t' | b' ' | b'[' => return None,
            _ if size > 999 => return None,
            b']' if size == 0 => return None,
            b']' => return Some(index),
            b'\\'
                if index + 1 < end && matches!(bytes[index + 1], b'[' | b'\\' | b']') =>
            {
                index += 2;
                size += 2;
            }
            _ => {
                index += 1;
                size += 1;
            }
        }
    }
}

/// A GFM table head row from `at` to `end`: its cells as micromark counts them.
pub(crate) fn scan_head_row(bytes: &[u8], at: usize, end: usize) -> Option<usize> {
    let (mut size, mut size_b, mut seen) = (0, 0, false);
    let mut index = at;
    if index < end && bytes[index] != b'|' {
        seen = true;
        // The first character that is not a pipe counts double
        size_b += 1;
    }
    while index < end {
        match bytes[index] {
            b' ' | b'\t' => index += 1,
            byte => {
                size_b += 1;
                if seen {
                    seen = false;
                    size += 1;
                }
                if byte == b'|' {
                    seen = true;
                    index += 1;
                } else {
                    while index < end && !matches!(bytes[index], b' ' | b'\t' | b'|') {
                        if bytes[index] == b'\\'
                            && index + 1 < end
                            && matches!(bytes[index + 1], b'\\' | b'|')
                        {
                            index += 1;
                        }
                        index += 1;
                    }
                }
            }
        }
    }
    // Anything but one pipe, whitespace aside, makes a head row
    (size_b > 1).then_some(size)
}

/// A GFM table delimiter row from `at` to `end`: its cells' alignments.
pub(crate) fn scan_delimiter_row(bytes: &[u8], at: usize, end: usize) -> Option<usize> {
    scan_delimiter_aligns(bytes, at, end).map(|aligns| aligns.len())
}

/// Each cell of a delimiter row: whether it starts and ends with `:`.
pub(crate) fn scan_delimiter_aligns(
    bytes: &[u8],
    at: usize,
    end: usize,
) -> Option<Vec<(bool, bool)>> {
    let mut index = at;
    let mut seen = false;
    let mut cells = vec![];
    let skip = |mut index: usize| {
        while index < end && is_space_or_tab(bytes[index]) {
            index += 1;
        }
        index
    };
    loop {
        // Before a cell: a pipe, or a value
        if index < end && bytes[index] == b'|' {
            seen = true;
            index = skip(index + 1);
            if index >= end {
                break;
            }
        } else if !(index < end && matches!(bytes[index], b'-' | b':')) {
            return None;
        }
        let left = bytes[index] == b':';
        if left {
            seen = true;
            index += 1;
        }
        if index >= end || bytes[index] != b'-' {
            return None;
        }
        while index < end && bytes[index] == b'-' {
            index += 1;
        }
        let right = index < end && bytes[index] == b':';
        if right {
            seen = true;
            index += 1;
        }
        cells.push((left, right));
        index = skip(index);
        if index >= end {
            break;
        }
        if bytes[index] != b'|' {
            return None;
        }
    }
    // No `:` or `|` at all is a thematic break or setext underline instead
    (seen && !cells.is_empty()).then_some(cells)
}

/// A closing fence of colons from `at`: its size when only whitespace follows it, else 0.
pub(crate) fn scan_colon_fence(bytes: &[u8], at: usize, end: usize) -> usize {
    let mut index = at;
    while index < end && bytes[index] == b':' {
        index += 1;
    }
    if is_blank_from(bytes, index, end) {
        index - at
    } else {
        0
    }
}
