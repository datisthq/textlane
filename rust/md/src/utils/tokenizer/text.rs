use crate::actions::attention::tokenize::tokenize_attention;
use crate::actions::autolink::tokenize::tokenize_autolink;
use crate::actions::character_escape::tokenize::tokenize_character_escape;
use crate::actions::character_reference::tokenize::tokenize_character_reference;
use crate::actions::code_text::tokenize::tokenize_code_text;
use crate::actions::data::tokenize::tokenize_data;
use crate::actions::directive_text::tokenize::tokenize_directive_text;
use crate::actions::gfm_autolink_literal::tokenize::tokenize_gfm_autolink_literal;
use crate::actions::gfm_footnote_call::tokenize::tokenize_gfm_footnote_call;
use crate::actions::gfm_potential_footnote_call::tokenize::tokenize_gfm_potential_footnote_call;
use crate::actions::gfm_strikethrough::tokenize::tokenize_gfm_strikethrough;
use crate::actions::gfm_task_list_item_check::tokenize::tokenize_gfm_task_list_item_check;
use crate::actions::hard_break_escape::tokenize::tokenize_hard_break_escape;
use crate::actions::html_text::tokenize::tokenize_html_text;
use crate::actions::label_end::tokenize::tokenize_label_end;
use crate::actions::label_start_image::tokenize::tokenize_label_start_image;
use crate::actions::label_start_link::tokenize::tokenize_label_start_link;
use crate::actions::math_text::tokenize::tokenize_math_text;
use crate::actions::mdx_expression_text::tokenize::tokenize_mdx_expression_text;
use crate::actions::mdx_jsx_text::tokenize::tokenize_mdx_jsx_text;
use crate::helpers::literal::can_start_literal;
use crate::models::construct::{Resolver, State};
use crate::utils::tokenizer::driver::Tokenizer;

// Bytes that may start a text construct, where data stops to let one try
const MARKERS: [u8; 9] = *b"!&*<[\\]_`";
const MARKERS_GFM: [u8; 10] = *b"!&*<[\\]_`~";
const MARKERS_DIRECTIVE: [u8; 10] = *b"!&*:<[\\]_`";
const MARKERS_GFM_DIRECTIVE: [u8; 11] = *b"!&*:<[\\]_`~";

/// The text content type: phrasing, such as the inside of a paragraph or heading.
pub(crate) fn tokenize_text(tokenizer: &mut Tokenizer) -> State {
    let gfm = tokenizer.parse_state.gfm.is_some();
    tokenizer.tokenize_state.markers = match (gfm, tokenizer.parse_state.directive) {
        (false, false) => &MARKERS,
        (true, false) => &MARKERS_GFM,
        (false, true) => &MARKERS_DIRECTIVE,
        (true, true) => &MARKERS_GFM_DIRECTIVE,
    };
    tokenizer.tokenize_state.literal_autolinks = gfm;
    tokenizer.tokenize_state.math_text = tokenizer.parse_state.math.is_some();
    tokenizer.tokenize_state.mdx_text = tokenizer.parse_state.mdx.is_some();
    State::Retry(before)
}

// GFM's literal autolinks go first, as micromark tries extensions before the core
fn before(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.tokenize_state.literal_autolinks
        && let Some(byte) = tokenizer.current
        && can_start_literal(tokenizer.previous, byte)
    {
        tokenizer.attempt(State::Next(before), State::Next(before_construct));
        return State::Retry(tokenize_gfm_autolink_literal);
    }
    State::Retry(before_construct)
}

fn before_construct(tokenizer: &mut Tokenizer) -> State {
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
        Some(b'<') if tokenizer.tokenize_state.mdx_text => {
            tokenizer.attempt(State::Next(before), State::Next(before_data));
            State::Retry(tokenize_mdx_jsx_text)
        }
        Some(b'{') if tokenizer.tokenize_state.mdx_text => {
            tokenizer.attempt(State::Next(before), State::Next(before_data));
            State::Retry(tokenize_mdx_expression_text)
        }
        Some(b'<') => {
            tokenizer.attempt(State::Next(before), State::Next(before_html));
            State::Retry(tokenize_autolink)
        }
        // micromark tries GFM's footnote call, then its task list check, before a link
        Some(b'[') => {
            let next = if tokenizer.tokenize_state.task_list_check_allowed {
                before_gfm_task_list_item_check
            } else {
                before_label_start_link
            };
            tokenizer.attempt(State::Next(before), State::Next(next));
            State::Retry(tokenize_gfm_footnote_call)
        }
        Some(b']') => {
            tokenizer.attempt(
                State::Next(before),
                State::Next(before_gfm_potential_footnote_call),
            );
            State::Retry(tokenize_label_end)
        }
        Some(b'&') => {
            tokenizer.attempt(State::Next(before), State::Next(before_data));
            State::Retry(tokenize_character_reference)
        }
        Some(b':') if tokenizer.parse_state.directive => {
            tokenizer.attempt(State::Next(before), State::Next(before_data));
            State::Retry(tokenize_directive_text)
        }
        Some(b'*' | b'_') => {
            tokenizer.attempt(State::Next(before), State::Next(before_data));
            State::Retry(tokenize_attention)
        }
        Some(b'~') if tokenizer.parse_state.gfm.is_some() => {
            tokenizer.attempt(State::Next(before), State::Next(before_data));
            State::Retry(tokenize_gfm_strikethrough)
        }
        Some(b'`') => {
            tokenizer.attempt(State::Next(before), State::Next(before_data));
            State::Retry(tokenize_code_text)
        }
        Some(b'$') if tokenizer.tokenize_state.math_text => {
            tokenizer.attempt(State::Next(before), State::Next(before_data));
            State::Retry(tokenize_math_text)
        }
        Some(b'\\') => {
            tokenizer.attempt(State::Next(before), State::Next(before_hard_break_escape));
            State::Retry(tokenize_character_escape)
        }
        _ => State::Retry(before_data),
    }
}

fn before_label_start_link(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(before), State::Next(before_data));
    State::Retry(tokenize_label_start_link)
}

fn before_html(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(before), State::Next(before_data));
    State::Retry(tokenize_html_text)
}

fn before_gfm_task_list_item_check(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(before), State::Next(before_label_start_link));
    State::Retry(tokenize_gfm_task_list_item_check)
}

fn before_gfm_potential_footnote_call(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(before), State::Next(before_data));
    State::Retry(tokenize_gfm_potential_footnote_call)
}

fn before_hard_break_escape(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(before), State::Next(before_data));
    State::Retry(tokenize_hard_break_escape)
}

fn before_data(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(before), State::Nok);
    State::Retry(tokenize_data)
}
