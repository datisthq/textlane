use crate::actions::data::tokenize::tokenize_data;
use crate::models::construct::{Resolver, State};
use crate::utils::tokenizer::driver::Tokenizer;

// Bytes that may start a text construct, where data stops to let one try
const MARKERS: [u8; 0] = [];

/// The text content type: phrasing, such as the inside of a paragraph or heading.
pub(crate) fn tokenize_text(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.markers = &MARKERS;
    State::Retry(before)
}

fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.register_resolver(Resolver::Data);
            tokenizer.register_resolver(Resolver::Text);
            State::Ok
        }
        _ => State::Retry(before_data),
    }
}

fn before_data(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(before), State::Nok);
    State::Retry(tokenize_data)
}
