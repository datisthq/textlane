use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// A backslash at the end of a line: a hard break.
pub(crate) fn tokenize_hard_break_escape(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'\\') {
        return State::Nok;
    }
    tokenizer.enter(Name::HARD_BREAK_ESCAPE);
    tokenizer.consume();
    State::Next(after)
}

fn after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => {
            tokenizer.exit(Name::HARD_BREAK_ESCAPE);
            State::Ok
        }
        _ => State::Nok,
    }
}
