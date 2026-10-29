use crate::actions::space_or_tab::tokenize::{
    tokenize_space_or_tab, tokenize_space_or_tab_indent,
};
use crate::models::construct::{Resolver, State};
use crate::models::event::{ContentType, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const OPENING_SEQUENCE_SIZE_MAX: usize = 6;

/// An ATX heading: one to six `#`, the text, and an optional closing sequence.
pub(crate) fn tokenize_heading_atx(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::HEADING_ATX);
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(before), State::Nok);
        State::Retry(tokenize_space_or_tab_indent(tokenizer, 0))
    } else {
        State::Retry(before)
    }
}

fn before(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'#') {
        tokenizer.enter(Name::HEADING_ATX_SEQUENCE);
        State::Retry(sequence_open)
    } else {
        State::Nok
    }
}

fn sequence_open(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'#')
        && tokenizer.tokenize_state.size < OPENING_SEQUENCE_SIZE_MAX
    {
        tokenizer.tokenize_state.size += 1;
        tokenizer.consume();
        State::Next(sequence_open)
    } else if matches!(tokenizer.current, None | Some(b'\t' | b'\n' | b' ')) {
        tokenizer.tokenize_state.size = 0;
        tokenizer.exit(Name::HEADING_ATX_SEQUENCE);
        State::Retry(at_break)
    } else {
        tokenizer.tokenize_state.size = 0;
        State::Nok
    }
}

fn at_break(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::HEADING_ATX);
            tokenizer.register_resolver(Resolver::HeadingAtx);
            tokenizer.interrupt = false;
            State::Ok
        }
        Some(b'\t' | b' ') => {
            tokenizer.attempt(State::Next(at_break), State::Nok);
            State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
        }
        Some(b'#') => {
            tokenizer.enter(Name::HEADING_ATX_SEQUENCE);
            State::Retry(sequence_further)
        }
        Some(_) => {
            tokenizer.enter_link(
                Name::DATA,
                Link {
                    previous: None,
                    next: None,
                    content: ContentType::Text,
                },
            );
            State::Retry(data)
        }
    }
}

fn sequence_further(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'#') {
        tokenizer.consume();
        State::Next(sequence_further)
    } else {
        tokenizer.exit(Name::HEADING_ATX_SEQUENCE);
        State::Retry(at_break)
    }
}

fn data(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\t' | b'\n' | b' ') => {
            tokenizer.exit(Name::DATA);
            State::Retry(at_break)
        }
        _ => {
            tokenizer.consume();
            State::Next(data)
        }
    }
}
