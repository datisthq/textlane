use crate::models::options::Gemoji;
use crate::plugins::gemoji::emoticons::EMOTICONS;
use crate::plugins::gemoji::names::NAMES;

/// A replacement, labelled for `accessible`.
pub(crate) struct Piece {
    pub(crate) text: String,
    pub(crate) label: Option<String>,
}

/// What a text becomes: one text again, as find-and-replace merges what it splits, holding
/// the first replacement as its `<span>` when that one opened the text.
pub(crate) struct Replaced {
    pub(crate) value: String,
    pub(crate) lead: Option<Piece>,
}

/// Replace each `:name:` gemoji knows with its emoji, as remark-gemoji's
/// `/:(\+1|[-\w]+):/g` finds them; `None` when nothing is replaced.
pub(crate) fn replace_shortcodes(value: &str) -> Option<String> {
    let found = find_shortcodes(value);
    if found.is_empty() {
        return None;
    }
    let mut result = String::with_capacity(value.len());
    let mut copied = 0;
    for (start, end, emoji) in found {
        result.push_str(&value[copied..start]);
        result.push_str(emoji);
        copied = end;
    }
    result.push_str(&value[copied..]);
    Some(result)
}

/// remark-emoji's two find-and-replace passes: shortcodes, then, with `emoticon`, emoticons
/// in the text the first left; `None` when nothing is replaced.
pub(crate) fn replace_emoji(value: &str, options: &Gemoji) -> Option<Replaced> {
    let pad = if options.pad_space_after { " " } else { "" };
    let shortcodes: Vec<(usize, usize, Piece)> = find_shortcodes(value)
        .into_iter()
        .map(|(start, end, emoji)| {
            let label = options
                .accessible
                .then(|| format!("{} emoji", read_label(&value[start + 1..end - 1])));
            let text = format!("{emoji}{pad}");
            (start, end, Piece { text, label })
        })
        .collect();
    let first = splice(value, shortcodes);
    if !options.emoticon {
        return first;
    }
    let text = first.as_ref().map_or(value, |first| first.value.as_str());
    match splice(text, find_emoticons(text, options)) {
        Some(second) => Some(second),
        None => first,
    }
}

// mdast-util-find-and-replace's `add`: every piece is text, so each joins the one before,
// whose `data` the joined text keeps; only a replacement at the start brings its own
fn splice(value: &str, found: Vec<(usize, usize, Piece)>) -> Option<Replaced> {
    if found.is_empty() {
        return None;
    }
    let mut result = String::with_capacity(value.len());
    let mut lead = None;
    let mut copied = 0;
    for (index, (start, end, piece)) in found.into_iter().enumerate() {
        result.push_str(&value[copied..start]);
        result.push_str(&piece.text);
        if index == 0 && start == 0 && piece.label.is_some() {
            lead = Some(piece);
        }
        copied = end;
    }
    result.push_str(&value[copied..]);
    Some(Replaced {
        value: result,
        lead,
    })
}

// The shortcodes gemoji knows, each `(start, end, emoji)`; an unknown name is retried from
// its next character, so its closing colon can open the next shortcode
fn find_shortcodes(value: &str) -> Vec<(usize, usize, &'static str)> {
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

// `name.replace(/(?:_|-(?!1))/g, ' ')`
fn read_label(name: &str) -> String {
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

// `/(^|\s)[@$|*'",;.=:\-)([\]\\/<>038BOopPsSdDxXzZ]{2,5}/g` over one text, each match
// the replacer refuses retried one character on, as find-and-replace does
fn find_emoticons(text: &str, options: &Gemoji) -> Vec<(usize, usize, Piece)> {
    let characters: Vec<(usize, char)> = text.char_indices().collect();
    let byte = |index: usize| characters.get(index).map_or(text.len(), |(at, _)| *at);
    let run = |from: usize| {
        characters[from.min(characters.len())..]
            .iter()
            .take(5)
            .take_while(|(_, character)| is_emoticon_character(*character))
            .count()
    };
    let mut found = Vec::new();
    let mut search = 0;
    while let Some((start, end)) = (search..characters.len()).find_map(|index| {
        if index == 0 && run(0) >= 2 {
            return Some((0, run(0)));
        }
        (is_js_space(characters[index].1) && run(index + 1) >= 2)
            .then(|| (index, index + 1 + run(index + 1)))
    }) {
        match replace_emoticon(&text[byte(start)..byte(end)], options) {
            Some(piece) => {
                found.push((byte(start), byte(end), piece));
                search = end;
            }
            None => search = start + 1,
        }
    }
    found
}

// remark-emoji's `replaceEmoticon`: the whole match, or it without its last, first or both
// end characters, which stay beside the emoji
fn replace_emoticon(found: &str, options: &Gemoji) -> Option<Piece> {
    let first = found.chars().next().map_or(0, char::len_utf8);
    let last = found.chars().next_back().map_or(0, char::len_utf8);
    let find = |text: &str| {
        EMOTICONS
            .binary_search_by(|(each, _, _)| each.cmp(&text))
            .ok()
            .map(|index| EMOTICONS[index])
    };
    let full = find(found);
    let part_start = find(&found[..found.len() - last]);
    let part_end = find(&found[first..]);
    let part = (found.len() > first + last)
        .then(|| find(&found[first..found.len() - last]))
        .flatten();
    let (_, name, emoji) = full.or(part_start).or(part_end).or(part)?;
    let trimmed = if full.is_none()
        && part_end.is_none()
        && (part.is_some() || part_start.is_some())
    {
        &found[found.len() - last..]
    } else {
        ""
    };
    let start_trimmed = if full.is_none()
        && part_start.is_none()
        && (part.is_some() || part_end.is_some())
    {
        &found[..first]
    } else {
        ""
    };
    let pad = if options.pad_space_after { " " } else { "" };
    Some(Piece {
        text: format!("{start_trimmed}{emoji}{pad}{trimmed}"),
        label: options.accessible.then(|| format!("{name} emoticon")),
    })
}

fn is_emoticon_character(character: char) -> bool {
    "@$|*'\",;.=:-)([]\\/<>038BOopPsSdDxXzZ".contains(character)
}

// JavaScript's `\s`
fn is_js_space(character: char) -> bool {
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
