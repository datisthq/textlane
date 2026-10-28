use crate::models::construct::{State, StateFn};
use crate::models::event::{ContentType, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;
use crate::utils::tokenizer::subtokenize::link;

/// Between `min` and `max` spaces or tabs; returns the state to retry into.
pub(crate) fn tokenize_space_or_tab(
    tokenizer: &mut Tokenizer,
    min: usize,
    max: usize,
) -> StateFn {
    tokenize_space_or_tab_linked(tokenizer, min, max, None, false)
}

/// Between `min` and `max` spaces or tabs, as a token named `name`.
pub(crate) fn tokenize_space_or_tab_named(
    tokenizer: &mut Tokenizer,
    name: Name,
    min: usize,
    max: usize,
) -> StateFn {
    let state = tokenize_space_or_tab_linked(tokenizer, min, max, None, false);
    tokenizer.tokenize_state.space_or_tab_token = Some(name);
    state
}

/// Spaces or tabs that are a chunk of `content`, linked to the chunk before when `connect`.
pub(crate) fn tokenize_space_or_tab_linked(
    tokenizer: &mut Tokenizer,
    min: usize,
    max: usize,
    content: Option<ContentType>,
    connect: bool,
) -> StateFn {
    tokenizer.tokenize_state.space_or_tab_connect = connect;
    tokenizer.tokenize_state.space_or_tab_content = content;
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
        match tokenizer.tokenize_state.space_or_tab_content {
            Some(content) => {
                tokenizer.enter_link(
                    name,
                    Link {
                        previous: None,
                        next: None,
                        content,
                    },
                );
                if tokenizer.tokenize_state.space_or_tab_connect {
                    let index = tokenizer.events.len() - 1;
                    link(&mut tokenizer.events, index);
                } else {
                    tokenizer.tokenize_state.space_or_tab_connect = true;
                }
            }
            None => tokenizer.enter(name),
        }
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
