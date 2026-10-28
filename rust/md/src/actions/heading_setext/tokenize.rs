use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab;
use crate::helpers::skip::skip_opt_back;
use crate::models::construct::{Resolver, State};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const TAB_SIZE: usize = 4;

/// A setext heading's underline: `=` or `-` under a paragraph, which becomes the heading.
pub(crate) fn tokenize_heading_setext(tokenizer: &mut Tokenizer) -> State {
    let after_content = tokenizer.events.len().checked_sub(1).is_some_and(|last| {
        let before = skip_opt_back(
            &tokenizer.events,
            last,
            &[Name::LINE_ENDING, Name::SPACE_OR_TAB],
        );
        let name = tokenizer.events[before].name;
        name == Name::CONTENT || name == Name::HEADING_SETEXT_UNDERLINE
    });
    if tokenizer.lazy || tokenizer.pierce || !after_content {
        return State::Nok;
    }
    tokenizer.enter(Name::HEADING_SETEXT_UNDERLINE);
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(before), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 0, TAB_SIZE - 1))
    } else {
        State::Retry(before)
    }
}

fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(marker @ (b'-' | b'=')) => {
            tokenizer.tokenize_state.marker = marker;
            tokenizer.enter(Name::HEADING_SETEXT_UNDERLINE_SEQUENCE);
            State::Retry(inside)
        }
        _ => State::Nok,
    }
}

fn inside(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(tokenizer.tokenize_state.marker) {
        tokenizer.consume();
        return State::Next(inside);
    }
    tokenizer.tokenize_state.marker = 0;
    tokenizer.exit(Name::HEADING_SETEXT_UNDERLINE_SEQUENCE);
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(after), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
    } else {
        State::Retry(after)
    }
}

fn after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.interrupt = false;
            tokenizer.register_resolver(Resolver::HeadingSetext);
            tokenizer.exit(Name::HEADING_SETEXT_UNDERLINE);
            State::Ok
        }
        _ => State::Nok,
    }
}
