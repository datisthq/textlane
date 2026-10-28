use crate::actions::non_lazy_continuation::tokenize::tokenize_non_lazy_continuation;
use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab;
use crate::helpers::slice::{Slice, span_of_exit};
use crate::models::construct::State;
use crate::models::event::{ContentType, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const TAB_SIZE: usize = 4;
const SEQUENCE_SIZE_MIN: usize = 2;

/// Math (flow): an opening fence of two or more `$` with an optional meta, the math, and
/// a closing fence at least as long.
pub(crate) fn tokenize_math_flow(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.math.is_none() {
        return State::Nok;
    }
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(before_sequence_open), State::Nok);
        return State::Retry(tokenize_space_or_tab(tokenizer, 0, TAB_SIZE - 1));
    }
    if tokenizer.current == Some(b'$') {
        return State::Retry(before_sequence_open);
    }
    State::Nok
}

fn before_sequence_open(tokenizer: &mut Tokenizer) -> State {
    // The opening fence's indent is removed from each line of the math
    let mut prefix = 0;
    if let Some(event) = tokenizer.events.last()
        && event.name == Name::SPACE_OR_TAB
    {
        let (start, end) = span_of_exit(&tokenizer.events, tokenizer.events.len() - 1);
        prefix = Slice::from_points(tokenizer.parse_state.bytes, start, end).len();
    }
    if tokenizer.current != Some(b'$') {
        return State::Nok;
    }
    tokenizer.tokenize_state.size_c = prefix;
    tokenizer.enter(Name::MATH_FLOW);
    tokenizer.enter(Name::MATH_FLOW_FENCE);
    tokenizer.enter(Name::MATH_FLOW_FENCE_SEQUENCE);
    State::Retry(sequence_open)
}

fn sequence_open(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'$') {
        tokenizer.tokenize_state.size += 1;
        tokenizer.consume();
        return State::Next(sequence_open);
    }
    if tokenizer.tokenize_state.size < SEQUENCE_SIZE_MIN {
        return reset(tokenizer);
    }
    tokenizer.exit(Name::MATH_FLOW_FENCE_SEQUENCE);
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(meta_before), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
    } else {
        State::Retry(meta_before)
    }
}

fn meta_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => State::Retry(meta_after),
        _ => {
            tokenizer.enter(Name::MATH_FLOW_FENCE_META);
            tokenizer.enter_link(
                Name::DATA,
                Link {
                    previous: None,
                    next: None,
                    content: ContentType::String,
                },
            );
            State::Retry(meta)
        }
    }
}

fn meta(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::DATA);
            tokenizer.exit(Name::MATH_FLOW_FENCE_META);
            State::Retry(meta_after)
        }
        Some(b'$') => reset(tokenizer),
        Some(_) => {
            tokenizer.consume();
            State::Next(meta)
        }
    }
}

fn meta_after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(Name::MATH_FLOW_FENCE);
    // A fence is a whole construct: lazy lines cannot continue a paragraph past it
    tokenizer.concrete = true;
    tokenizer.check(State::Next(at_non_lazy_break), State::Next(after));
    State::Retry(tokenize_non_lazy_continuation)
}

fn at_non_lazy_break(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Next(content_before));
    tokenizer.enter(Name::LINE_ENDING);
    tokenizer.consume();
    tokenizer.exit(Name::LINE_ENDING);
    State::Next(close_start)
}

fn close_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::MATH_FLOW_FENCE);
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(before_sequence_close), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 0, TAB_SIZE - 1))
    } else {
        State::Retry(before_sequence_close)
    }
}

fn before_sequence_close(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'$') {
        tokenizer.enter(Name::MATH_FLOW_FENCE_SEQUENCE);
        State::Retry(sequence_close)
    } else {
        State::Nok
    }
}

fn sequence_close(tokenizer: &mut Tokenizer) -> State {
    let state = &mut tokenizer.tokenize_state;
    if tokenizer.current == Some(b'$') {
        state.size_b += 1;
        tokenizer.consume();
        return State::Next(sequence_close);
    }
    if state.size_b < state.size {
        state.size_b = 0;
        return State::Nok;
    }
    state.size_b = 0;
    tokenizer.exit(Name::MATH_FLOW_FENCE_SEQUENCE);
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(sequence_close_after), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
    } else {
        State::Retry(sequence_close_after)
    }
}

fn sequence_close_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::MATH_FLOW_FENCE);
            State::Ok
        }
        _ => State::Nok,
    }
}

fn content_before(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::LINE_ENDING);
    tokenizer.consume();
    tokenizer.exit(Name::LINE_ENDING);
    State::Next(content_start)
}

fn content_start(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(before_content_chunk), State::Nok);
        let prefix = tokenizer.tokenize_state.size_c;
        State::Retry(tokenize_space_or_tab(tokenizer, 0, prefix))
    } else {
        State::Retry(before_content_chunk)
    }
}

fn before_content_chunk(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.check(State::Next(at_non_lazy_break), State::Next(after));
            State::Retry(tokenize_non_lazy_continuation)
        }
        _ => {
            tokenizer.enter(Name::MATH_FLOW_VALUE);
            State::Retry(content_chunk)
        }
    }
}

fn content_chunk(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::MATH_FLOW_VALUE);
            State::Retry(before_content_chunk)
        }
        _ => {
            tokenizer.consume();
            State::Next(content_chunk)
        }
    }
}

fn after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(Name::MATH_FLOW);
    tokenizer.tokenize_state.size_c = 0;
    tokenizer.tokenize_state.size = 0;
    tokenizer.interrupt = false;
    tokenizer.concrete = false;
    State::Ok
}

fn reset(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.size_c = 0;
    tokenizer.tokenize_state.size = 0;
    State::Nok
}
