use crate::models::construct::State;
use crate::models::event::{Event, Point};
use crate::utils::tokenizer::document::tokenize_document;
use crate::utils::tokenizer::driver::Tokenizer;
use crate::utils::tokenizer::subtokenize::subtokenize;

/// What every tokenizer of one document shares.
#[derive(Debug)]
pub(crate) struct ParseState<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) definitions: Vec<String>,
}

/// Tokenize a document into its events, subtokenizing until no linked content is left.
pub(crate) fn tokenize_input(value: &str) -> Vec<Event> {
    let mut parse_state = ParseState {
        bytes: value.as_bytes(),
        definitions: vec![],
    };
    let start = Point {
        line: 1,
        column: 1,
        index: 0,
        vs: 0,
    };
    let (mut events, mut result) = {
        let mut tokenizer = Tokenizer::new(start, &parse_state);
        let state = tokenizer.push(
            (0, 0),
            (parse_state.bytes.len(), 0),
            State::Next(tokenize_document),
        );
        let result = tokenizer.flush(state, true);
        (tokenizer.events, result)
    };
    loop {
        parse_state.definitions.append(&mut result.definitions);
        if result.done {
            return events;
        }
        result = subtokenize(&mut events, &parse_state, None);
    }
}
