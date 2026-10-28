use crate::models::construct::State;
use crate::models::event::{ContentType, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const SIZE_MAX: usize = 999;

/// A link label, `[a]`, of at most 999 bytes with something other than whitespace; its
/// tokens are `token_1` (the whole), `token_2` (the brackets) and `token_3` (the string).
pub(crate) fn tokenize_label(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'[') {
        return State::Nok;
    }
    let state = &tokenizer.tokenize_state;
    let (whole, marker, string) = (state.token_1, state.token_2, state.token_3);
    tokenizer.enter(whole);
    tokenizer.enter(marker);
    tokenizer.consume();
    tokenizer.exit(marker);
    tokenizer.enter(string);
    State::Next(at_break)
}

fn at_break(tokenizer: &mut Tokenizer) -> State {
    let state = &tokenizer.tokenize_state;
    if state.size > SIZE_MAX
        || matches!(tokenizer.current, None | Some(b'['))
        || (tokenizer.current == Some(b']') && !state.seen)
    {
        return State::Retry(nok);
    }
    match tokenizer.current {
        // As micromark: the line ending is not string content, but the indent after it is
        Some(b'\n') => {
            tokenizer.enter(Name::LINE_ENDING);
            tokenizer.consume();
            tokenizer.exit(Name::LINE_ENDING);
            State::Next(at_break)
        }
        Some(b']') => {
            let state = &tokenizer.tokenize_state;
            let (whole, marker, string) = (state.token_1, state.token_2, state.token_3);
            tokenizer.exit(string);
            tokenizer.enter(marker);
            tokenizer.consume();
            tokenizer.exit(marker);
            tokenizer.exit(whole);
            reset(tokenizer);
            State::Ok
        }
        _ => {
            // Each line is a string of its own, as micromark's chunks are
            tokenizer.enter_link(
                Name::DATA,
                Link {
                    previous: None,
                    next: None,
                    content: ContentType::String,
                },
            );
            State::Retry(inside)
        }
    }
}

fn nok(tokenizer: &mut Tokenizer) -> State {
    reset(tokenizer);
    State::Nok
}

fn reset(tokenizer: &mut Tokenizer) {
    tokenizer.tokenize_state.connect = false;
    tokenizer.tokenize_state.seen = false;
    tokenizer.tokenize_state.size = 0;
}

fn inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n' | b'[' | b']') => {
            tokenizer.exit(Name::DATA);
            State::Retry(at_break)
        }
        Some(_) if tokenizer.tokenize_state.size > SIZE_MAX => {
            tokenizer.exit(Name::DATA);
            State::Retry(at_break)
        }
        Some(byte) => {
            tokenizer.consume();
            tokenizer.tokenize_state.size += 1;
            if !matches!(byte, b'\t' | b' ') {
                tokenizer.tokenize_state.seen = true;
            }
            State::Next(if byte == b'\\' { escape } else { inside })
        }
    }
}

fn escape(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'[' | b'\\' | b']') => {
            tokenizer.consume();
            tokenizer.tokenize_state.size += 1;
            State::Next(inside)
        }
        _ => State::Retry(inside),
    }
}
