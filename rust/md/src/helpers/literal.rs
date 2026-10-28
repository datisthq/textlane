use crate::helpers::character::{
    CharacterKind, classify_character, classify_character_unit, find_character_after,
};

/// GFM's email atext: `+`, `-`, `.`, `_` and ASCII alphanumerics.
pub(crate) fn is_atext(byte: u8) -> bool {
    matches!(byte, b'+' | b'-' | b'.' | b'_') || byte.is_ascii_alphanumeric()
}

/// Whether a GFM literal autolink may start at `byte` after `previous`: micromark stops
/// data only where an email, `www.` or protocol literal passes its `previous` check.
pub(crate) fn can_start_literal(previous: Option<u8>, byte: u8) -> bool {
    can_start_email(previous, byte)
        || can_start_www(previous, byte)
        || can_start_protocol(previous, byte)
}

/// An email literal starts at atext that does not follow atext or `/`.
pub(crate) fn can_start_email(previous: Option<u8>, byte: u8) -> bool {
    is_atext(byte)
        && previous.is_none_or(|previous| previous != b'/' && !is_atext(previous))
}

/// A `www.` literal starts after whitespace, `(`, `*`, `_`, `[`, `]` or `~`.
pub(crate) fn can_start_www(previous: Option<u8>, byte: u8) -> bool {
    matches!(byte, b'W' | b'w')
        && matches!(
            previous,
            None | Some(b'\t' | b'\n' | b' ' | b'(' | b'*' | b'_' | b'[' | b']' | b'~')
        )
}

/// A protocol literal starts after anything but an ASCII letter.
pub(crate) fn can_start_protocol(previous: Option<u8>, byte: u8) -> bool {
    matches!(byte, b'H' | b'h')
        && !previous.is_some_and(|byte| byte.is_ascii_alphabetic())
}

/// Classify the character at `index`, where the tokenizer's `current` is `byte`, as
/// micromark does a UTF-16 code unit.
pub(crate) fn classify_unit_at(
    bytes: &[u8],
    index: usize,
    byte: Option<u8>,
) -> CharacterKind {
    match byte {
        None => CharacterKind::Whitespace,
        Some(byte) if byte.is_ascii() => classify_character(Some(char::from(byte))),
        Some(_) => match find_character_after(bytes, index) {
            Some(character) => classify_character_unit(Some(character)),
            None => CharacterKind::Other,
        },
    }
}
