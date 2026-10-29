use crate::helpers::character::{
    CharacterKind, classify_character, classify_character_unit, find_character_after,
};
use crate::models::construct::State;
use crate::utils::tokenizer::driver::Tokenizer;

/// A directive's name: a character that is neither punctuation nor whitespace, then more
/// of those, `-` and `_`, not ending in `-` or `_`.
pub(crate) fn tokenize_directive_name(tokenizer: &mut Tokenizer) -> State {
    if classify_directive_character(tokenizer) != CharacterKind::Other {
        return State::Nok;
    }
    let name = tokenizer.tokenize_state.directive.kind.names().name;
    tokenizer.enter(name);
    tokenizer.consume();
    State::Next(inside)
}

fn inside(tokenizer: &mut Tokenizer) -> State {
    let other = classify_directive_character(tokenizer) == CharacterKind::Other;
    if other || matches!(tokenizer.current, Some(b'-' | b'_')) {
        tokenizer.consume();
        return State::Next(inside);
    }
    let name = tokenizer.tokenize_state.directive.kind.names().name;
    tokenizer.exit(name);
    if matches!(tokenizer.previous, Some(b'-' | b'_')) {
        State::Nok
    } else {
        State::Ok
    }
}

/// Classify the current character as micromark does a code: the end and line endings are
/// whitespace; a tab, its virtual spaces and the inside of a multibyte character are
/// neither, as micromark's negative codes for tabs are not.
pub(crate) fn classify_directive_character(tokenizer: &Tokenizer) -> CharacterKind {
    match tokenizer.current {
        None => CharacterKind::Whitespace,
        Some(b'\t') => CharacterKind::Other,
        Some(b' ') if tokenizer.point.vs > 0 => CharacterKind::Other,
        Some(byte) if byte.is_ascii() => classify_character(Some(char::from(byte))),
        Some(0x80..=0xBF) => CharacterKind::Other,
        Some(_) => classify_character_unit(find_character_after(
            tokenizer.parse_state.bytes,
            tokenizer.point.index as usize,
        )),
    }
}
