use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab;
use crate::models::construct::State;
use crate::models::event::{ContentType, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// A title in `"…"`, `'…'` or `(…)`; tokens `token_1` (the whole), `token_2` (the
/// markers) and `token_3` (the string).
pub(crate) fn tokenize_title(tokenizer: &mut Tokenizer) -> State {
    let Some(marker @ (b'"' | b'\'' | b'(')) = tokenizer.current else {
        return State::Nok;
    };
    tokenizer.tokenize_state.marker = if marker == b'(' { b')' } else { marker };
    let state = &tokenizer.tokenize_state;
    let (whole, markers) = (state.token_1, state.token_2);
    tokenizer.enter(whole);
    tokenizer.enter(markers);
    tokenizer.consume();
    tokenizer.exit(markers);
    State::Next(begin)
}

fn begin(tokenizer: &mut Tokenizer) -> State {
    let state = &tokenizer.tokenize_state;
    let (whole, markers, string) = (state.token_1, state.token_2, state.token_3);
    if tokenizer.current == Some(state.marker) {
        tokenizer.enter(markers);
        tokenizer.consume();
        tokenizer.exit(markers);
        tokenizer.exit(whole);
        tokenizer.tokenize_state.marker = 0;
        tokenizer.tokenize_state.connect = false;
        State::Ok
    } else {
        tokenizer.enter(string);
        State::Retry(at_break)
    }
}

fn at_break(tokenizer: &mut Tokenizer) -> State {
    let Some(byte) = tokenizer.current else {
        return State::Retry(nok);
    };
    if byte == tokenizer.tokenize_state.marker {
        let string = tokenizer.tokenize_state.token_3;
        tokenizer.exit(string);
        return State::Retry(begin);
    }
    // As micromark: neither the line ending nor the indent after it is string content
    if byte == b'\n' {
        tokenizer.enter(Name::LINE_ENDING);
        tokenizer.consume();
        tokenizer.exit(Name::LINE_ENDING);
        return State::Next(line_start);
    }
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

fn line_start(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(at_break), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
    } else {
        State::Retry(at_break)
    }
}

fn nok(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.marker = 0;
    tokenizer.tokenize_state.connect = false;
    State::Nok
}

fn inside(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(tokenizer.tokenize_state.marker)
        || matches!(tokenizer.current, None | Some(b'\n'))
    {
        tokenizer.exit(Name::DATA);
        return State::Retry(at_break);
    }
    let escaped = tokenizer.current == Some(b'\\');
    tokenizer.consume();
    State::Next(if escaped { escape } else { inside })
}

fn escape(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'"' | b'\'' | b')' | b'\\') => {
            tokenizer.consume();
            State::Next(inside)
        }
        _ => State::Retry(inside),
    }
}
