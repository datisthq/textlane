use crate::models::construct::State;
use crate::models::directive::CHUNK_TEXT;
use crate::models::event::{ContentType, Link};
use crate::utils::tokenizer::driver::Tokenizer;
use crate::utils::tokenizer::subtokenize::link_to;

const BALANCE_MAX: usize = 32;
const SIZE_MAX: usize = 999;

/// A directive's label, `[a]`: text, possibly empty, with balanced brackets nested at most
/// 32 deep; on one line unless the directive is a text one.
pub(crate) fn tokenize_directive_label(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'[') {
        return State::Nok;
    }
    let names = tokenizer.tokenize_state.directive.kind.names();
    let state = &mut tokenizer.tokenize_state.directive;
    state.balance = 0;
    state.label_size = 0;
    state.label_chunk = None;
    tokenizer.enter(names.label);
    tokenizer.enter(names.label_marker);
    tokenizer.consume();
    tokenizer.exit(names.label_marker);
    State::Next(after_start)
}

fn after_start(tokenizer: &mut Tokenizer) -> State {
    let names = tokenizer.tokenize_state.directive.kind.names();
    if tokenizer.current == Some(b']') {
        tokenizer.enter(names.label_marker);
        tokenizer.consume();
        tokenizer.exit(names.label_marker);
        tokenizer.exit(names.label);
        return State::Ok;
    }
    tokenizer.enter(names.label_string);
    State::Retry(line_start)
}

fn line_start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b']') && tokenizer.tokenize_state.directive.balance == 0
    {
        return State::Retry(at_closing);
    }
    let previous = tokenizer.tokenize_state.directive.label_chunk;
    tokenizer.enter_link(
        CHUNK_TEXT,
        Link {
            previous: None,
            next: None,
            content: ContentType::TextTrailing,
        },
    );
    let index = tokenizer.events.len() - 1;
    if let Some(previous) = previous {
        link_to(&mut tokenizer.events, previous, index);
    }
    tokenizer.tokenize_state.directive.label_chunk = Some(index);
    State::Retry(data)
}

fn data(tokenizer: &mut Tokenizer) -> State {
    let state = &mut tokenizer.tokenize_state.directive;
    match tokenizer.current {
        None => State::Nok,
        _ if state.label_size > SIZE_MAX => State::Nok,
        Some(b'[') => {
            state.balance += 1;
            if state.balance > BALANCE_MAX {
                return State::Nok;
            }
            tokenizer.consume();
            State::Next(data)
        }
        Some(b']') if state.balance == 0 => {
            tokenizer.exit(CHUNK_TEXT);
            State::Retry(at_closing)
        }
        Some(b'\n') => {
            if tokenizer.tokenize_state.directive.kind.names().disallow_eol {
                return State::Nok;
            }
            tokenizer.consume();
            tokenizer.exit(CHUNK_TEXT);
            State::Next(line_start)
        }
        Some(byte) => {
            if byte == b']' {
                state.balance -= 1;
            }
            tokenizer.consume();
            State::Next(if byte == b'\\' { data_escape } else { data })
        }
    }
}

fn data_escape(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'[' | b'\\' | b']')) {
        tokenizer.consume();
        tokenizer.tokenize_state.directive.label_size += 1;
        return State::Next(data);
    }
    State::Retry(data)
}

fn at_closing(tokenizer: &mut Tokenizer) -> State {
    let names = tokenizer.tokenize_state.directive.kind.names();
    tokenizer.exit(names.label_string);
    tokenizer.enter(names.label_marker);
    tokenizer.consume();
    tokenizer.exit(names.label_marker);
    tokenizer.exit(names.label);
    State::Ok
}
