use crate::helpers::byte_set::{ByteSet, has_byte};
use crate::helpers::literal::can_start_literal;
use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// Data: everything up to a marker another construct may start at, and line endings.
pub(crate) fn tokenize_data(tokenizer: &mut Tokenizer) -> State {
    if let Some(byte) = tokenizer.current
        && is_marker(tokenizer, byte)
    {
        tokenizer.enter(Name::DATA);
        tokenizer.consume();
        return State::Next(inside);
    }
    State::Retry(at_break)
}

fn at_break(tokenizer: &mut Tokenizer) -> State {
    if let Some(byte) = tokenizer.current
        && !is_marker(tokenizer, byte)
    {
        if byte == b'\n' {
            tokenizer.enter(Name::LINE_ENDING);
            tokenizer.consume();
            tokenizer.exit(Name::LINE_ENDING);
            return State::Next(at_break);
        }
        tokenizer.enter(Name::DATA);
        return State::Retry(inside);
    }
    State::Ok
}

fn inside(tokenizer: &mut Tokenizer) -> State {
    if let Some(byte) = tokenizer.current
        && byte != b'\n'
        && !is_marker(tokenizer, byte)
    {
        let markers = tokenizer.tokenize_state.markers;
        let literal = tokenizer.tokenize_state.literal_autolinks;
        tokenizer.consume_while(|previous, byte| {
            !is_marker_after(&markers, literal, previous, byte)
        });
        return State::Next(inside);
    }
    tokenizer.exit(Name::DATA);
    State::Retry(at_break)
}

fn is_marker(tokenizer: &Tokenizer, byte: u8) -> bool {
    let state = &tokenizer.tokenize_state;
    is_marker_after(
        &state.markers,
        state.literal_autolinks,
        tokenizer.previous,
        byte,
    )
}

// GFM's literal autolinks start at letters too, but only where their previous byte allows
fn is_marker_after(
    markers: &ByteSet,
    literal: bool,
    previous: Option<u8>,
    byte: u8,
) -> bool {
    has_byte(markers, byte) || (literal && can_start_literal(previous, byte))
}
