use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// Math (text): a run of dollars, the math, and a run of as many dollars; one dollar only
/// with `singleDollarTextMath`.
pub(crate) fn tokenize_math_text(tokenizer: &mut Tokenizer) -> State {
    // A dollar right after another one is part of that run, unless the first was escaped
    let after_escape = tokenizer
        .events
        .last()
        .is_some_and(|event| event.name == Name::CHARACTER_ESCAPE);
    if tokenizer.current != Some(b'$')
        || (tokenizer.previous == Some(b'$') && !after_escape)
    {
        return State::Nok;
    }
    tokenizer.enter(Name::MATH_TEXT);
    tokenizer.enter(Name::MATH_TEXT_SEQUENCE);
    State::Retry(sequence_open)
}

fn sequence_open(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'$') {
        tokenizer.tokenize_state.size += 1;
        tokenizer.consume();
        return State::Next(sequence_open);
    }
    let single = tokenizer
        .parse_state
        .math
        .as_ref()
        .is_some_and(|options| options.single_dollar_text_math);
    let state = &tokenizer.tokenize_state;
    // As code spans: once one scanned to the end, a run with none of its length after it
    // cannot close, and scanning again would make input quadratic
    let start = tokenizer.point.index - state.size;
    let closable = state
        .math_text_runs
        .get(&state.size)
        .is_some_and(|last| *last > start);
    if (state.size < 2 && !single) || (state.math_text_scanned && !closable) {
        tokenizer.tokenize_state.size = 0;
        return State::Nok;
    }
    tokenizer.exit(Name::MATH_TEXT_SEQUENCE);
    State::Retry(between)
}

fn between(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.tokenize_state.size = 0;
            tokenizer.tokenize_state.math_text_scanned = true;
            State::Nok
        }
        Some(b'\n') => {
            tokenizer.enter(Name::LINE_ENDING);
            tokenizer.consume();
            tokenizer.exit(Name::LINE_ENDING);
            State::Next(between)
        }
        Some(b'$') => {
            tokenizer.enter(Name::MATH_TEXT_SEQUENCE);
            State::Retry(sequence_close)
        }
        _ => {
            tokenizer.enter(Name::MATH_TEXT_DATA);
            State::Retry(data)
        }
    }
}

fn data(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n' | b'$') => {
            tokenizer.exit(Name::MATH_TEXT_DATA);
            State::Retry(between)
        }
        _ => {
            tokenizer.consume();
            State::Next(data)
        }
    }
}

fn sequence_close(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'$') {
        tokenizer.tokenize_state.size_b += 1;
        tokenizer.consume();
        return State::Next(sequence_close);
    }
    let size = tokenizer.tokenize_state.size_b;
    let start = tokenizer.point.index - size;
    let last = tokenizer
        .tokenize_state
        .math_text_runs
        .entry(size)
        .or_insert(start);
    *last = (*last).max(start);
    tokenizer.exit(Name::MATH_TEXT_SEQUENCE);
    if tokenizer.tokenize_state.size == tokenizer.tokenize_state.size_b {
        tokenizer.exit(Name::MATH_TEXT);
        tokenizer.tokenize_state.size = 0;
        tokenizer.tokenize_state.size_b = 0;
        State::Ok
    } else {
        // A run of another length is math, not the closing sequence
        let len = tokenizer.events.len();
        tokenizer.events[len - 2].name = Name::MATH_TEXT_DATA;
        tokenizer.events[len - 1].name = Name::MATH_TEXT_DATA;
        tokenizer.tokenize_state.size_b = 0;
        State::Retry(between)
    }
}
