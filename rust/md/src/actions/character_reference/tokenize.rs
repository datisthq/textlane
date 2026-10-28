use crate::helpers::entities::decode_named;
use crate::helpers::reference::{accept_reference_byte, measure_reference};
use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// A character reference: `&amp;`, `&#123;` or `&#x7B;`.
pub(crate) fn tokenize_character_reference(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'&') {
        return State::Nok;
    }
    tokenizer.enter(Name::CHARACTER_REFERENCE);
    tokenizer.enter(Name::CHARACTER_REFERENCE_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::CHARACTER_REFERENCE_MARKER);
    State::Next(open)
}

fn open(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'#') {
        tokenizer.enter(Name::CHARACTER_REFERENCE_MARKER_NUMERIC);
        tokenizer.consume();
        tokenizer.exit(Name::CHARACTER_REFERENCE_MARKER_NUMERIC);
        State::Next(numeric)
    } else {
        tokenizer.tokenize_state.marker = b'&';
        tokenizer.enter(Name::CHARACTER_REFERENCE_VALUE);
        State::Retry(value)
    }
}

fn numeric(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'x' | b'X') = tokenizer.current {
        tokenizer.enter(Name::CHARACTER_REFERENCE_MARKER_HEXADECIMAL);
        tokenizer.consume();
        tokenizer.exit(Name::CHARACTER_REFERENCE_MARKER_HEXADECIMAL);
        tokenizer.enter(Name::CHARACTER_REFERENCE_VALUE);
        tokenizer.tokenize_state.marker = b'x';
        State::Next(value)
    } else {
        tokenizer.enter(Name::CHARACTER_REFERENCE_VALUE);
        tokenizer.tokenize_state.marker = b'#';
        State::Retry(value)
    }
}

fn value(tokenizer: &mut Tokenizer) -> State {
    let marker = tokenizer.tokenize_state.marker;
    let size = tokenizer.tokenize_state.size;
    if tokenizer.current == Some(b';') && size > 0 {
        // A name must be a known one; a number is always decoded
        if marker == b'&' {
            let end = tokenizer.point.index;
            let name = tokenizer
                .parse_state
                .bytes
                .get(end - size..end)
                .unwrap_or_default();
            let known = std::str::from_utf8(name)
                .ok()
                .and_then(decode_named)
                .is_some();
            if !known {
                return reset(tokenizer);
            }
        }
        tokenizer.exit(Name::CHARACTER_REFERENCE_VALUE);
        tokenizer.enter(Name::CHARACTER_REFERENCE_MARKER_SEMI);
        tokenizer.consume();
        tokenizer.exit(Name::CHARACTER_REFERENCE_MARKER_SEMI);
        tokenizer.exit(Name::CHARACTER_REFERENCE);
        tokenizer.tokenize_state.marker = 0;
        tokenizer.tokenize_state.size = 0;
        return State::Ok;
    }
    if let Some(byte) = tokenizer.current
        && size < measure_reference(marker)
        && accept_reference_byte(marker, byte)
    {
        tokenizer.tokenize_state.size += 1;
        tokenizer.consume();
        return State::Next(value);
    }
    reset(tokenizer)
}

fn reset(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.marker = 0;
    tokenizer.tokenize_state.size = 0;
    State::Nok
}
