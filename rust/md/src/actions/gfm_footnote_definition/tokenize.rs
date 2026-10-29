use crate::actions::blank_line::tokenize::tokenize_blank_line;
use crate::actions::space_or_tab::tokenize::{
    tokenize_space_or_tab, tokenize_space_or_tab_indent,
};
use crate::helpers::identifier::normalize_identifier;
use crate::models::construct::State;
use crate::models::event::{ContentType, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const TAB_SIZE: usize = 4;
const SIZE_MAX: usize = 999;

/// A GFM footnote definition, `[^a]:`, then its content.
pub(crate) fn tokenize_gfm_footnote_definition(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.gfm.is_none() {
        return State::Nok;
    }
    tokenizer.enter(Name::GFM_FOOTNOTE_DEFINITION);
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(label_before), State::Nok);
        State::Retry(tokenize_space_or_tab_indent(tokenizer, 1))
    } else {
        State::Retry(label_before)
    }
}

/// The indent that continues a footnote definition on a later line, or a blank line.
pub(crate) fn continue_gfm_footnote_definition(tokenizer: &mut Tokenizer) -> State {
    tokenizer.check(State::Next(continue_blank), State::Next(continue_filled));
    State::Retry(tokenize_blank_line)
}

fn label_before(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'[') {
        return State::Nok;
    }
    tokenizer.enter(Name::GFM_FOOTNOTE_DEFINITION_LABEL);
    tokenizer.enter(Name::GFM_FOOTNOTE_DEFINITION_LABEL_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::GFM_FOOTNOTE_DEFINITION_LABEL_MARKER);
    State::Next(label_at_marker)
}

fn label_at_marker(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'^') {
        return State::Nok;
    }
    tokenizer.enter(Name::GFM_FOOTNOTE_DEFINITION_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::GFM_FOOTNOTE_DEFINITION_MARKER);
    tokenizer.enter(Name::GFM_FOOTNOTE_DEFINITION_LABEL_STRING);
    tokenizer.enter_link(
        Name::DATA,
        Link {
            previous: None,
            next: None,
            content: ContentType::String,
        },
    );
    tokenizer.tokenize_state.start = tokenizer.point.index;
    State::Next(label_inside)
}

fn label_inside(tokenizer: &mut Tokenizer) -> State {
    let size = tokenizer.tokenize_state.size;
    match tokenizer.current {
        None | Some(b'\t' | b'\n' | b' ' | b'[') => label_nok(tokenizer),
        _ if size > SIZE_MAX => label_nok(tokenizer),
        Some(b']') if size == 0 => label_nok(tokenizer),
        Some(b']') => {
            tokenizer.tokenize_state.size = 0;
            tokenizer.tokenize_state.end = tokenizer.point.index;
            tokenizer.exit(Name::DATA);
            tokenizer.exit(Name::GFM_FOOTNOTE_DEFINITION_LABEL_STRING);
            tokenizer.enter(Name::GFM_FOOTNOTE_DEFINITION_LABEL_MARKER);
            tokenizer.consume();
            tokenizer.exit(Name::GFM_FOOTNOTE_DEFINITION_LABEL_MARKER);
            tokenizer.exit(Name::GFM_FOOTNOTE_DEFINITION_LABEL);
            State::Next(label_after)
        }
        Some(byte) => {
            tokenizer.consume();
            tokenizer.tokenize_state.size += 1;
            State::Next(if byte == b'\\' {
                label_escape
            } else {
                label_inside
            })
        }
    }
}

fn label_nok(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.size = 0;
    State::Nok
}

fn label_escape(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'[' | b'\\' | b']') => {
            tokenizer.consume();
            tokenizer.tokenize_state.size += 1;
            State::Next(label_inside)
        }
        _ => State::Retry(label_inside),
    }
}

fn label_after(tokenizer: &mut Tokenizer) -> State {
    let state = &mut tokenizer.tokenize_state;
    let (start, end) = (state.start, state.end);
    state.start = 0;
    state.end = 0;
    if tokenizer.current != Some(b':') {
        return State::Nok;
    }
    let raw = &tokenizer.parse_state.bytes[start..end];
    let label = normalize_identifier(&String::from_utf8_lossy(raw));
    tokenizer
        .tokenize_state
        .gfm_footnote_definitions
        .push(label);
    tokenizer.enter(Name::DEFINITION_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::DEFINITION_MARKER);
    State::Next(whitespace_after)
}

fn whitespace_after(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Ok, State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
    } else {
        State::Ok
    }
}

fn continue_blank(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        State::Retry(tokenize_space_or_tab(tokenizer, 0, TAB_SIZE))
    } else {
        State::Ok
    }
}

fn continue_filled(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        State::Retry(tokenize_space_or_tab(tokenizer, TAB_SIZE, TAB_SIZE))
    } else {
        State::Nok
    }
}
