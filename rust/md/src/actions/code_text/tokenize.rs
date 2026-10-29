use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// A code span: a run of backticks, the code, and a run of as many backticks.
pub(crate) fn tokenize_code_text(tokenizer: &mut Tokenizer) -> State {
    // A backtick right after another one is part of that run, unless the first was escaped
    let after_escape = tokenizer
        .events
        .last()
        .is_some_and(|event| event.name == Name::CHARACTER_ESCAPE);
    if tokenizer.current != Some(b'`')
        || (tokenizer.previous == Some(b'`') && !after_escape)
    {
        return State::Nok;
    }
    tokenizer.enter(Name::CODE_TEXT);
    tokenizer.enter(Name::CODE_TEXT_SEQUENCE);
    State::Retry(sequence_open)
}

fn sequence_open(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'`') {
        tokenizer.tokenize_state.size += 1;
        tokenizer.consume();
        State::Next(sequence_open)
    } else {
        // After one span scanned to the end, every run is known: one with no run of its
        // length after it cannot close, and scanning again would make input quadratic
        let state = &tokenizer.tokenize_state;
        let start = tokenizer.point.index - state.size as u32;
        let closable = state
            .code_text_runs
            .get(&state.size)
            .is_some_and(|last| *last > start as usize);
        if state.code_text_scanned && !closable {
            tokenizer.tokenize_state.size = 0;
            return State::Nok;
        }
        tokenizer.exit(Name::CODE_TEXT_SEQUENCE);
        State::Retry(between)
    }
}

fn between(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.tokenize_state.size = 0;
            tokenizer.tokenize_state.code_text_scanned = true;
            State::Nok
        }
        Some(b'\n') => {
            tokenizer.enter(Name::LINE_ENDING);
            tokenizer.consume();
            tokenizer.exit(Name::LINE_ENDING);
            State::Next(between)
        }
        Some(b'`') => {
            tokenizer.enter(Name::CODE_TEXT_SEQUENCE);
            State::Retry(sequence_close)
        }
        _ => {
            tokenizer.enter(Name::CODE_TEXT_DATA);
            State::Retry(data)
        }
    }
}

fn data(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n' | b'`') => {
            tokenizer.exit(Name::CODE_TEXT_DATA);
            State::Retry(between)
        }
        _ => {
            tokenizer.consume();
            State::Next(data)
        }
    }
}

fn sequence_close(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'`') {
        tokenizer.tokenize_state.size_b += 1;
        tokenizer.consume();
        return State::Next(sequence_close);
    }
    let size = tokenizer.tokenize_state.size_b;
    let start = tokenizer.point.index - size as u32;
    // Spans closing early meet runs again: keep the last of each length, not the latest seen
    let last = tokenizer
        .tokenize_state
        .code_text_runs
        .entry(size)
        .or_insert(start as usize);
    *last = (*last).max(start as usize);
    tokenizer.exit(Name::CODE_TEXT_SEQUENCE);
    if tokenizer.tokenize_state.size == tokenizer.tokenize_state.size_b {
        tokenizer.exit(Name::CODE_TEXT);
        tokenizer.tokenize_state.size = 0;
        tokenizer.tokenize_state.size_b = 0;
        State::Ok
    } else {
        // A run of another length is code, not the closing sequence
        let len = tokenizer.events.len();
        tokenizer.events[len - 2].name = Name::CODE_TEXT_DATA;
        tokenizer.events[len - 1].name = Name::CODE_TEXT_DATA;
        tokenizer.tokenize_state.size_b = 0;
        State::Retry(between)
    }
}
