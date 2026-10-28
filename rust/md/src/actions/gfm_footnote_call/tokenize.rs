use crate::helpers::identifier::normalize_identifier;
use crate::models::construct::State;
use crate::models::event::{ContentType, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const SIZE_MAX: usize = 999;

/// `[^a]`: a call to a GFM footnote, when a definition of its label exists.
pub(crate) fn tokenize_gfm_footnote_call(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.gfm.is_none() || tokenizer.current != Some(b'[') {
        return State::Nok;
    }
    tokenizer.enter(Name::GFM_FOOTNOTE_CALL);
    tokenizer.enter(Name::GFM_FOOTNOTE_CALL_LABEL_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::GFM_FOOTNOTE_CALL_LABEL_MARKER);
    State::Next(call_start)
}

fn call_start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'^') {
        return State::Nok;
    }
    tokenizer.enter(Name::GFM_FOOTNOTE_CALL_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::GFM_FOOTNOTE_CALL_MARKER);
    tokenizer.enter(Name::GFM_FOOTNOTE_CALL_STRING);
    tokenizer.enter_link(
        Name::DATA,
        Link {
            previous: None,
            next: None,
            content: ContentType::String,
        },
    );
    tokenizer.tokenize_state.start = tokenizer.point.index;
    State::Next(call_data)
}

fn call_data(tokenizer: &mut Tokenizer) -> State {
    let size = tokenizer.tokenize_state.size;
    match tokenizer.current {
        None | Some(b'\t' | b'\n' | b' ' | b'[') => nok(tokenizer),
        _ if size > SIZE_MAX => nok(tokenizer),
        Some(b']') if size == 0 => nok(tokenizer),
        Some(b']') => {
            let state = &mut tokenizer.tokenize_state;
            let start = state.start;
            state.start = 0;
            state.size = 0;
            let raw = &tokenizer.parse_state.bytes[start..tokenizer.point.index];
            let label = normalize_identifier(&String::from_utf8_lossy(raw));
            if !tokenizer
                .parse_state
                .gfm_footnote_definitions
                .contains(&label)
            {
                return State::Nok;
            }
            tokenizer.exit(Name::DATA);
            tokenizer.exit(Name::GFM_FOOTNOTE_CALL_STRING);
            tokenizer.enter(Name::GFM_FOOTNOTE_CALL_LABEL_MARKER);
            tokenizer.consume();
            tokenizer.exit(Name::GFM_FOOTNOTE_CALL_LABEL_MARKER);
            tokenizer.exit(Name::GFM_FOOTNOTE_CALL);
            tokenizer.tokenize_state.gfm_footnote_image = None;
            State::Ok
        }
        Some(byte) => {
            tokenizer.consume();
            tokenizer.tokenize_state.size += 1;
            State::Next(if byte == b'\\' {
                call_escape
            } else {
                call_data
            })
        }
    }
}

fn call_escape(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'[' | b'\\' | b']') => {
            tokenizer.consume();
            tokenizer.tokenize_state.size += 1;
            State::Next(call_data)
        }
        _ => State::Retry(call_data),
    }
}

fn nok(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.start = 0;
    tokenizer.tokenize_state.size = 0;
    State::Nok
}
