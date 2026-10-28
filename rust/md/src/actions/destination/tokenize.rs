use crate::models::construct::State;
use crate::models::event::{ContentType, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// A destination, `<a b>` or `a(b)`; tokens `token_1` (the whole), `token_2` (enclosed),
/// `token_3` (its `<` and `>`), `token_4` (raw) and `token_5` (the string). Parentheses in a
/// raw destination nest at most `size_b` deep.
pub(crate) fn tokenize_destination(tokenizer: &mut Tokenizer) -> State {
    let state = &tokenizer.tokenize_state;
    let (whole, enclosed, marker, raw, string) = (
        state.token_1,
        state.token_2,
        state.token_3,
        state.token_4,
        state.token_5,
    );
    match tokenizer.current {
        Some(b'<') => {
            tokenizer.enter(whole);
            tokenizer.enter(enclosed);
            tokenizer.enter(marker);
            tokenizer.consume();
            tokenizer.exit(marker);
            State::Next(enclosed_before)
        }
        None | Some(0x01..=0x1F | b' ' | b')' | 0x7F) => State::Nok,
        Some(_) => {
            tokenizer.enter(whole);
            tokenizer.enter(raw);
            tokenizer.enter(string);
            enter_data(tokenizer);
            State::Retry(raw_inside)
        }
    }
}

fn enter_data(tokenizer: &mut Tokenizer) {
    tokenizer.enter_link(
        Name::DATA,
        Link {
            previous: None,
            next: None,
            content: ContentType::String,
        },
    );
}

fn enclosed_before(tokenizer: &mut Tokenizer) -> State {
    let state = &tokenizer.tokenize_state;
    let (whole, enclosed, marker, string) =
        (state.token_1, state.token_2, state.token_3, state.token_5);
    if tokenizer.current == Some(b'>') {
        tokenizer.enter(marker);
        tokenizer.consume();
        tokenizer.exit(marker);
        tokenizer.exit(enclosed);
        tokenizer.exit(whole);
        State::Ok
    } else {
        tokenizer.enter(string);
        enter_data(tokenizer);
        State::Retry(enclosed_inside)
    }
}

fn enclosed_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n' | b'<') => State::Nok,
        Some(b'>') => {
            tokenizer.exit(Name::DATA);
            let string = tokenizer.tokenize_state.token_5;
            tokenizer.exit(string);
            State::Retry(enclosed_before)
        }
        Some(byte) => {
            tokenizer.consume();
            State::Next(if byte == b'\\' {
                enclosed_escape
            } else {
                enclosed_inside
            })
        }
    }
}

fn enclosed_escape(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'<' | b'>' | b'\\') => {
            tokenizer.consume();
            State::Next(enclosed_inside)
        }
        _ => State::Retry(enclosed_inside),
    }
}

fn raw_inside(tokenizer: &mut Tokenizer) -> State {
    let state = &tokenizer.tokenize_state;
    if state.size == 0
        && matches!(tokenizer.current, None | Some(b'\t' | b'\n' | b' ' | b')'))
    {
        let (whole, raw, string) = (state.token_1, state.token_4, state.token_5);
        tokenizer.exit(Name::DATA);
        tokenizer.exit(string);
        tokenizer.exit(raw);
        tokenizer.exit(whole);
        tokenizer.tokenize_state.size = 0;
        return State::Ok;
    }
    match tokenizer.current {
        Some(b'(') if state.size < state.size_b => {
            tokenizer.consume();
            tokenizer.tokenize_state.size += 1;
            State::Next(raw_inside)
        }
        Some(b')') => {
            tokenizer.consume();
            tokenizer.tokenize_state.size -= 1;
            State::Next(raw_inside)
        }
        None | Some(0x01..=0x1F | b' ' | b'(' | 0x7F) => {
            tokenizer.tokenize_state.size = 0;
            State::Nok
        }
        Some(byte) => {
            tokenizer.consume();
            State::Next(if byte == b'\\' {
                raw_escape
            } else {
                raw_inside
            })
        }
    }
}

fn raw_escape(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'(' | b')' | b'\\') => {
            tokenizer.consume();
            State::Next(raw_inside)
        }
        _ => State::Retry(raw_inside),
    }
}
