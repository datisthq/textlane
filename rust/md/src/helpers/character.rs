use crate::helpers::unicode::is_unicode_punctuation;

/// What a character next to an emphasis marker is, which decides if the marker opens or closes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CharacterKind {
    Whitespace,
    Punctuation,
    Other,
}

/// Classify a character; the edge of the input counts as whitespace.
pub(crate) fn classify_character(character: Option<char>) -> CharacterKind {
    match character {
        None => CharacterKind::Whitespace,
        Some(character) if character.is_whitespace() => CharacterKind::Whitespace,
        Some(character)
            if character.is_ascii_punctuation() || is_unicode_punctuation(character) =>
        {
            CharacterKind::Punctuation
        }
        Some(_) => CharacterKind::Other,
    }
}

/// The character that ends right before byte `index`.
pub(crate) fn find_character_before(bytes: &[u8], index: usize) -> Option<char> {
    let start = index.saturating_sub(4);
    String::from_utf8_lossy(&bytes[start..index]).chars().last()
}

/// The character that starts at byte `index`.
pub(crate) fn find_character_after(bytes: &[u8], index: usize) -> Option<char> {
    let end = (index + 4).min(bytes.len());
    String::from_utf8_lossy(&bytes[index..end]).chars().next()
}
