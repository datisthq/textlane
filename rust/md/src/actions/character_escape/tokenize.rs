use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// A backslash before ASCII punctuation, which then counts as plain text.
pub(crate) fn tokenize_character_escape(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'\\') {
        return State::Nok;
    }
    tokenizer.enter(Name::CHARACTER_ESCAPE);
    tokenizer.enter(Name::CHARACTER_ESCAPE_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::CHARACTER_ESCAPE_MARKER);
    State::Next(inside)
}

fn inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'!'..=b'/' | b':'..=b'@' | b'['..=b'`' | b'{'..=b'~') => {
            tokenizer.enter(Name::CHARACTER_ESCAPE_VALUE);
            tokenizer.consume();
            tokenizer.exit(Name::CHARACTER_ESCAPE_VALUE);
            tokenizer.exit(Name::CHARACTER_ESCAPE);
            State::Ok
        }
        _ => State::Nok,
    }
}
