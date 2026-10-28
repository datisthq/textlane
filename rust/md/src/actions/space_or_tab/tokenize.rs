use crate::models::construct::{State, StateFn};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// Between `min` and `max` spaces or tabs; returns the state to retry into.
pub(crate) fn tokenize_space_or_tab(
    tokenizer: &mut Tokenizer,
    min: usize,
    max: usize,
) -> StateFn {
    tokenizer.tokenize_state.space_or_tab_connect = false;
    tokenizer.tokenize_state.space_or_tab_content = None;
    tokenizer.tokenize_state.space_or_tab_min = min;
    tokenizer.tokenize_state.space_or_tab_max = max;
    tokenizer.tokenize_state.space_or_tab_token = None;
    start
}

fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.tokenize_state.space_or_tab_max > 0
        && matches!(tokenizer.current, Some(b'\t' | b' '))
    {
        let name = tokenizer
            .tokenize_state
            .space_or_tab_token
            .unwrap_or(Name::SPACE_OR_TAB);
        tokenizer.enter(name);
        State::Retry(inside)
    } else {
        State::Retry(after)
    }
}

fn inside(tokenizer: &mut Tokenizer) -> State {
    let state = &tokenizer.tokenize_state;
    match tokenizer.current {
        Some(b'\t' | b' ') if state.space_or_tab_size < state.space_or_tab_max => {
            tokenizer.consume();
            tokenizer.tokenize_state.space_or_tab_size += 1;
            State::Next(inside)
        }
        _ => {
            let name = state.space_or_tab_token.unwrap_or(Name::SPACE_OR_TAB);
            tokenizer.exit(name);
            State::Retry(after)
        }
    }
}

fn after(tokenizer: &mut Tokenizer) -> State {
    let state = &mut tokenizer.tokenize_state;
    let result = if state.space_or_tab_size >= state.space_or_tab_min {
        State::Ok
    } else {
        State::Nok
    };
    state.space_or_tab_connect = false;
    state.space_or_tab_content = None;
    state.space_or_tab_size = 0;
    state.space_or_tab_max = 0;
    state.space_or_tab_min = 0;
    state.space_or_tab_token = None;
    result
}
