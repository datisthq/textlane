use crate::actions::attention::tokenize::tokenize_attention;
use crate::actions::autolink::tokenize::tokenize_autolink;
use crate::actions::character_escape::tokenize::tokenize_character_escape;
use crate::actions::character_reference::tokenize::tokenize_character_reference;
use crate::actions::code_text::tokenize::tokenize_code_text;
use crate::actions::data::tokenize::tokenize_data;
use crate::actions::hard_break_escape::tokenize::tokenize_hard_break_escape;
use crate::actions::html_text::tokenize::tokenize_html_text;
use crate::actions::label_end::tokenize::tokenize_label_end;
use crate::actions::label_start_image::tokenize::tokenize_label_start_image;
use crate::actions::label_start_link::tokenize::tokenize_label_start_link;
use crate::models::construct::{Resolver, State};
use crate::utils::tokenizer::driver::Tokenizer;

// Bytes that may start a text construct, where data stops to let one try
const MARKERS: [u8; 9] = *b"!&*<[\\]_`";

/// The text content type: phrasing, such as the inside of a paragraph or heading.
pub(crate) fn tokenize_text(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.markers = &MARKERS;
    State::Retry(before)
}

fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.register_resolver(Resolver::Data);
            tokenizer.register_resolver(Resolver::Text);
            State::Ok
        }
        Some(b'!') => {
            tokenizer.attempt(State::Next(before), State::Next(before_data));
            State::Retry(tokenize_label_start_image)
        }
        Some(b'<') => {
            tokenizer.attempt(State::Next(before), State::Next(before_html));
            State::Retry(tokenize_autolink)
        }
        Some(b'[') => {
            tokenizer.attempt(State::Next(before), State::Next(before_data));
            State::Retry(tokenize_label_start_link)
        }
        Some(b']') => {
            tokenizer.attempt(State::Next(before), State::Next(before_data));
            State::Retry(tokenize_label_end)
        }
        Some(b'&') => {
            tokenizer.attempt(State::Next(before), State::Next(before_data));
            State::Retry(tokenize_character_reference)
        }
        Some(b'*' | b'_') => {
            tokenizer.attempt(State::Next(before), State::Next(before_data));
            State::Retry(tokenize_attention)
        }
        Some(b'`') => {
            tokenizer.attempt(State::Next(before), State::Next(before_data));
            State::Retry(tokenize_code_text)
        }
        Some(b'\\') => {
            tokenizer.attempt(State::Next(before), State::Next(before_hard_break_escape));
            State::Retry(tokenize_character_escape)
        }
        _ => State::Retry(before_data),
    }
}

fn before_html(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(before), State::Next(before_data));
    State::Retry(tokenize_html_text)
}

fn before_hard_break_escape(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(before), State::Next(before_data));
    State::Retry(tokenize_hard_break_escape)
}

fn before_data(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(before), State::Nok);
    State::Retry(tokenize_data)
}
