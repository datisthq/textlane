use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab_linked;
use crate::models::construct::{State, StateFn};
use crate::models::event::{ContentType, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;
use crate::utils::tokenizer::subtokenize::link;

/// Spaces or tabs with at most one line ending among them, as between a link's parts;
/// chunks of `content`, linked when `connect`, when given.
pub(crate) fn tokenize_space_or_tab_eol(
    tokenizer: &mut Tokenizer,
    content: Option<ContentType>,
    connect: bool,
) -> StateFn {
    tokenizer.tokenize_state.space_or_tab_eol_content = content;
    tokenizer.tokenize_state.space_or_tab_eol_connect = connect;
    start
}

fn start(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(after_first), State::Next(at_eol));
        let state = &tokenizer.tokenize_state;
        let (content, connect) = (
            state.space_or_tab_eol_content,
            state.space_or_tab_eol_connect,
        );
        State::Retry(tokenize_space_or_tab_linked(
            tokenizer,
            1,
            usize::MAX,
            content,
            connect,
        ))
    } else {
        State::Retry(at_eol)
    }
}

fn after_first(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.space_or_tab_eol_ok = true;
    State::Retry(at_eol)
}

fn at_eol(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'\n') {
        let state = &mut tokenizer.tokenize_state;
        let ok = state.space_or_tab_eol_ok;
        reset(tokenizer);
        return if ok { State::Ok } else { State::Nok };
    }
    let state = &tokenizer.tokenize_state;
    match state.space_or_tab_eol_content {
        Some(content) => tokenizer.enter_link(
            Name::LINE_ENDING,
            Link {
                previous: None,
                next: None,
                content,
            },
        ),
        None => tokenizer.enter(Name::LINE_ENDING),
    }
    if tokenizer.tokenize_state.space_or_tab_eol_connect {
        let index = tokenizer.events.len() - 1;
        link(&mut tokenizer.events, index);
    } else if tokenizer.tokenize_state.space_or_tab_eol_content.is_some() {
        tokenizer.tokenize_state.space_or_tab_eol_connect = true;
    }
    tokenizer.consume();
    tokenizer.exit(Name::LINE_ENDING);
    State::Next(after_eol)
}

fn after_eol(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(after_more), State::Nok);
        let state = &tokenizer.tokenize_state;
        let (content, connect) = (
            state.space_or_tab_eol_content,
            state.space_or_tab_eol_connect,
        );
        State::Retry(tokenize_space_or_tab_linked(
            tokenizer,
            1,
            usize::MAX,
            content,
            connect,
        ))
    } else {
        State::Retry(after_more)
    }
}

fn after_more(tokenizer: &mut Tokenizer) -> State {
    reset(tokenizer);
    State::Ok
}

fn reset(tokenizer: &mut Tokenizer) {
    let state = &mut tokenizer.tokenize_state;
    state.space_or_tab_eol_content = None;
    state.space_or_tab_eol_connect = false;
    state.space_or_tab_eol_ok = false;
}
