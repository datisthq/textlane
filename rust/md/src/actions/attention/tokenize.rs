use crate::models::construct::{Resolver, State};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// A sequence of `*` or `_`; the attention resolver later pairs sequences into emphasis.
pub(crate) fn tokenize_attention(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(marker @ (b'*' | b'_')) => {
            tokenizer.tokenize_state.marker = marker;
            tokenizer.enter(Name::ATTENTION_SEQUENCE);
            State::Retry(inside)
        }
        _ => State::Nok,
    }
}

fn inside(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(tokenizer.tokenize_state.marker) {
        tokenizer.consume();
        State::Next(inside)
    } else {
        tokenizer.exit(Name::ATTENTION_SEQUENCE);
        tokenizer.register_resolver(Resolver::Attention);
        tokenizer.tokenize_state.marker = 0;
        State::Ok
    }
}
