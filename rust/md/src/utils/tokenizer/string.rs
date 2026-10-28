use crate::actions::data::tokenize::tokenize_data;
use crate::models::construct::{Resolver, State};
use crate::utils::tokenizer::driver::Tokenizer;

// Bytes that may start a string construct: character references and escapes
const MARKERS: [u8; 2] = *b"&\\";

/// The string content type: text without phrasing, such as a link destination.
pub(crate) fn tokenize_string(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.markers = &MARKERS;
    State::Retry(before)
}

fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.register_resolver(Resolver::Data);
            tokenizer.register_resolver(Resolver::String);
            State::Ok
        }
        _ => State::Retry(before_data),
    }
}

fn before_data(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(before), State::Nok);
    State::Retry(tokenize_data)
}
