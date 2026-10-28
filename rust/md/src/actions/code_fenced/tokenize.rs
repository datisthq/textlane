use crate::actions::non_lazy_continuation::tokenize::tokenize_non_lazy_continuation;
use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab;
use crate::helpers::slice::{Slice, span_of_exit};
use crate::models::construct::State;
use crate::models::event::{ContentType, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const TAB_SIZE: usize = 4;
const SEQUENCE_SIZE_MIN: usize = 3;

/// Fenced code: an opening fence of three or more `` ` `` or `~` with an optional info
/// string, the code, and a closing fence at least as long.
pub(crate) fn tokenize_code_fenced(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(before_sequence_open), State::Nok);
        return State::Retry(tokenize_space_or_tab(tokenizer, 0, TAB_SIZE - 1));
    }
    if matches!(tokenizer.current, Some(b'`' | b'~')) {
        return State::Retry(before_sequence_open);
    }
    State::Nok
}

fn before_sequence_open(tokenizer: &mut Tokenizer) -> State {
    // The opening fence's indent is removed from each line of the code
    let mut prefix = 0;
    if let Some(event) = tokenizer.events.last()
        && event.name == Name::SPACE_OR_TAB
    {
        let (start, end) = span_of_exit(&tokenizer.events, tokenizer.events.len() - 1);
        prefix = Slice::from_points(tokenizer.parse_state.bytes, start, end).len();
    }
    let Some(marker @ (b'`' | b'~')) = tokenizer.current else {
        return State::Nok;
    };
    tokenizer.tokenize_state.marker = marker;
    tokenizer.tokenize_state.size_c = prefix;
    tokenizer.enter(Name::CODE_FENCED);
    tokenizer.enter(Name::CODE_FENCED_FENCE);
    tokenizer.enter(Name::CODE_FENCED_FENCE_SEQUENCE);
    State::Retry(sequence_open)
}

fn sequence_open(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(tokenizer.tokenize_state.marker) {
        tokenizer.tokenize_state.size += 1;
        tokenizer.consume();
        return State::Next(sequence_open);
    }
    if tokenizer.tokenize_state.size < SEQUENCE_SIZE_MIN {
        return reset(tokenizer);
    }
    tokenizer.exit(Name::CODE_FENCED_FENCE_SEQUENCE);
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(info_before), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
    } else {
        State::Retry(info_before)
    }
}

fn info_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::CODE_FENCED_FENCE);
            // A fence is a whole construct: lazy lines cannot continue a paragraph past it
            tokenizer.concrete = true;
            tokenizer.check(State::Next(at_non_lazy_break), State::Next(after));
            State::Retry(tokenize_non_lazy_continuation)
        }
        _ => {
            tokenizer.enter(Name::CODE_FENCED_FENCE_INFO);
            enter_string(tokenizer);
            State::Retry(info)
        }
    }
}

fn info(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::DATA);
            tokenizer.exit(Name::CODE_FENCED_FENCE_INFO);
            State::Retry(info_before)
        }
        Some(b'\t' | b' ') => {
            tokenizer.exit(Name::DATA);
            tokenizer.exit(Name::CODE_FENCED_FENCE_INFO);
            tokenizer.attempt(State::Next(meta_before), State::Nok);
            State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
        }
        // A backtick fence's info may not hold a backtick, or it would be a code span
        Some(b'`') if tokenizer.tokenize_state.marker == b'`' => {
            tokenizer.concrete = false;
            reset(tokenizer)
        }
        Some(_) => {
            tokenizer.consume();
            State::Next(info)
        }
    }
}

fn meta_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => State::Retry(info_before),
        _ => {
            tokenizer.enter(Name::CODE_FENCED_FENCE_META);
            enter_string(tokenizer);
            State::Retry(meta)
        }
    }
}

fn meta(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::DATA);
            tokenizer.exit(Name::CODE_FENCED_FENCE_META);
            State::Retry(info_before)
        }
        Some(b'`') if tokenizer.tokenize_state.marker == b'`' => {
            tokenizer.concrete = false;
            reset(tokenizer)
        }
        Some(_) => {
            tokenizer.consume();
            State::Next(meta)
        }
    }
}

fn enter_string(tokenizer: &mut Tokenizer) {
    tokenizer.enter_link(
        Name::DATA,
        Link {
            previous: None,
            next: None,
            content: ContentType::String,
        },
    );
}

fn at_non_lazy_break(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Next(content_before));
    tokenizer.enter(Name::LINE_ENDING);
    tokenizer.consume();
    tokenizer.exit(Name::LINE_ENDING);
    State::Next(close_start)
}

fn close_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::CODE_FENCED_FENCE);
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(before_sequence_close), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 0, TAB_SIZE - 1))
    } else {
        State::Retry(before_sequence_close)
    }
}

fn before_sequence_close(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(tokenizer.tokenize_state.marker) {
        tokenizer.enter(Name::CODE_FENCED_FENCE_SEQUENCE);
        State::Retry(sequence_close)
    } else {
        State::Nok
    }
}

fn sequence_close(tokenizer: &mut Tokenizer) -> State {
    let state = &mut tokenizer.tokenize_state;
    if tokenizer.current == Some(state.marker) {
        state.size_b += 1;
        tokenizer.consume();
        return State::Next(sequence_close);
    }
    if state.size_b < state.size {
        state.size_b = 0;
        return State::Nok;
    }
    state.size_b = 0;
    tokenizer.exit(Name::CODE_FENCED_FENCE_SEQUENCE);
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
            tokenizer.exit(Name::CODE_FENCED_FENCE);
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
            tokenizer.enter(Name::CODE_FLOW_CHUNK);
            State::Retry(content_chunk)
        }
    }
}

fn content_chunk(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::CODE_FLOW_CHUNK);
            State::Retry(before_content_chunk)
        }
        _ => {
            tokenizer.consume();
            State::Next(content_chunk)
        }
    }
}

fn after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(Name::CODE_FENCED);
    tokenizer.tokenize_state.marker = 0;
    tokenizer.tokenize_state.size_c = 0;
    tokenizer.tokenize_state.size = 0;
    tokenizer.interrupt = false;
    tokenizer.concrete = false;
    State::Ok
}

fn reset(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.marker = 0;
    tokenizer.tokenize_state.size_c = 0;
    tokenizer.tokenize_state.size = 0;
    State::Nok
}
