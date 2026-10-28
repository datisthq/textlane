use crate::actions::blank_line::tokenize::tokenize_blank_line;
use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab;
use crate::actions::thematic_break::tokenize::tokenize_thematic_break;
use crate::helpers::slice::Slice;
use crate::models::construct::{Resolver, State};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const TAB_SIZE: usize = 4;
const VALUE_SIZE_MAX: usize = 10;

/// A list item: a bullet (`*`, `+`, `-`) or a number and `.` or `)`, then its content.
pub(crate) fn tokenize_list_item(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::LIST_ITEM);
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(before), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 0, TAB_SIZE - 1))
    } else {
        State::Retry(before)
    }
}

/// The indent that continues a list item on a later line, or a blank line.
pub(crate) fn continue_list_item(tokenizer: &mut Tokenizer) -> State {
    tokenizer.check(State::Next(continue_blank), State::Next(continue_filled));
    State::Retry(tokenize_blank_line)
}

fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        // `*` and `-` may start a thematic break instead, which wins
        Some(b'*' | b'-') => {
            tokenizer.check(State::Nok, State::Next(before_unordered));
            State::Retry(tokenize_thematic_break)
        }
        Some(b'+') => State::Retry(before_unordered),
        // Only an item numbered 1 may interrupt a paragraph
        Some(b'1') => State::Retry(before_ordered),
        Some(b'0'..=b'9') if !tokenizer.interrupt => State::Retry(before_ordered),
        _ => State::Nok,
    }
}

fn before_unordered(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::LIST_ITEM_PREFIX);
    State::Retry(marker)
}

fn before_ordered(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::LIST_ITEM_PREFIX);
    tokenizer.enter(Name::LIST_ITEM_VALUE);
    State::Retry(value)
}

fn value(tokenizer: &mut Tokenizer) -> State {
    let size = tokenizer.tokenize_state.size;
    match tokenizer.current {
        Some(b'.' | b')') if !tokenizer.interrupt || size < 2 => {
            tokenizer.exit(Name::LIST_ITEM_VALUE);
            State::Retry(marker)
        }
        Some(b'0'..=b'9') if size + 1 < VALUE_SIZE_MAX => {
            tokenizer.tokenize_state.size += 1;
            tokenizer.consume();
            State::Next(value)
        }
        _ => {
            tokenizer.tokenize_state.size = 0;
            State::Nok
        }
    }
}

fn marker(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::LIST_ITEM_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::LIST_ITEM_MARKER);
    State::Next(marker_after)
}

fn marker_after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.size = 1;
    tokenizer.check(State::Next(after), State::Next(marker_after_filled));
    State::Retry(tokenize_blank_line)
}

fn marker_after_filled(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.size = 0;
    tokenizer.attempt(State::Next(after), State::Next(prefix_other));
    State::Retry(whitespace)
}

fn whitespace(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(whitespace_after), State::Nok);
    State::Retry(tokenize_space_or_tab(tokenizer, 1, TAB_SIZE))
}

fn whitespace_after(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'\t' | b' ') = tokenizer.current {
        State::Nok
    } else {
        State::Ok
    }
}

// Five or more spaces after the marker: one belongs to the prefix, the rest is indented code
fn prefix_other(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b' ') => {
            tokenizer.enter(Name::SPACE_OR_TAB);
            tokenizer.consume();
            tokenizer.exit(Name::SPACE_OR_TAB);
            State::Next(after)
        }
        _ => State::Nok,
    }
}

fn after(tokenizer: &mut Tokenizer) -> State {
    let blank = tokenizer.tokenize_state.size == 1;
    tokenizer.tokenize_state.size = 0;
    // An empty item cannot interrupt a paragraph
    if blank && tokenizer.interrupt {
        return State::Nok;
    }
    let start = tokenizer
        .events
        .iter()
        .rposition(|event| event.name == Name::LIST_ITEM)
        .unwrap_or_default();
    let start = tokenizer.events[start].point.clone();
    let mut prefix =
        Slice::from_points(tokenizer.parse_state.bytes, &start, &tokenizer.point).len();
    if blank {
        prefix += 1;
    }
    let continued = tokenizer.tokenize_state.document_continued;
    if let Some(container) = tokenizer
        .tokenize_state
        .document_container_stack
        .get_mut(continued)
    {
        container.blank_initial = blank;
        container.size = prefix;
    }
    tokenizer.exit(Name::LIST_ITEM_PREFIX);
    tokenizer.register_resolver_before(Resolver::ListItem);
    State::Ok
}

fn continue_blank(tokenizer: &mut Tokenizer) -> State {
    let continued = tokenizer.tokenize_state.document_continued;
    let Some(container) = tokenizer
        .tokenize_state
        .document_container_stack
        .get(continued)
    else {
        return State::Nok;
    };
    let size = container.size;
    // An item that started blank ends at the next blank line
    if container.blank_initial {
        State::Nok
    } else if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        State::Retry(tokenize_space_or_tab(tokenizer, 0, size))
    } else {
        State::Ok
    }
}

fn continue_filled(tokenizer: &mut Tokenizer) -> State {
    let continued = tokenizer.tokenize_state.document_continued;
    let Some(container) = tokenizer
        .tokenize_state
        .document_container_stack
        .get_mut(continued)
    else {
        return State::Nok;
    };
    let size = container.size;
    if container.blank_initial {
        container.blank_initial = false;
        tokenizer.tokenize_state.document_blank_ends = None;
    }
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        State::Retry(tokenize_space_or_tab(tokenizer, size, size))
    } else {
        State::Nok
    }
}
