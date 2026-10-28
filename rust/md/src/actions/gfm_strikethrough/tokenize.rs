use crate::models::construct::{Resolver, State};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// A sequence of one or two `~`; the strikethrough resolver later pairs sequences of the
/// same size.
pub(crate) fn tokenize_gfm_strikethrough(tokenizer: &mut Tokenizer) -> State {
    // A `~` right after another is part of a longer run, unless that one was escaped
    let escaped = tokenizer
        .events
        .last()
        .is_some_and(|event| event.name == Name::CHARACTER_ESCAPE);
    if tokenizer.current != Some(b'~') || (tokenizer.previous == Some(b'~') && !escaped) {
        return State::Nok;
    }
    tokenizer.enter(Name::STRIKETHROUGH_SEQUENCE_TEMPORARY);
    State::Retry(more)
}

fn more(tokenizer: &mut Tokenizer) -> State {
    let size = tokenizer.tokenize_state.size;
    if tokenizer.current == Some(b'~') {
        if size > 1 {
            tokenizer.tokenize_state.size = 0;
            return State::Nok;
        }
        tokenizer.tokenize_state.size += 1;
        tokenizer.consume();
        return State::Next(more);
    }
    tokenizer.tokenize_state.size = 0;
    let single = tokenizer
        .parse_state
        .gfm
        .as_ref()
        .is_none_or(|gfm| gfm.single_tilde);
    if size < 2 && !single {
        return State::Nok;
    }
    tokenizer.exit(Name::STRIKETHROUGH_SEQUENCE_TEMPORARY);
    tokenizer.register_resolver(Resolver::GfmStrikethrough);
    State::Ok
}
