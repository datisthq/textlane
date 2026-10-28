use crate::actions::blank_line::tokenize::tokenize_blank_line;
use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab;
use crate::helpers::indent::read_def_list_indent;
use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const TAB_SIZE: usize = 4;

/// A definition description: `:` and whitespace, then its content.
pub(crate) fn tokenize_def_list(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::DEF_LIST_DESCRIPTION);
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(marker), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 0, TAB_SIZE - 1))
    } else {
        State::Retry(marker)
    }
}

/// The indent that continues a definition description on a later line, or a blank line.
pub(crate) fn continue_def_list(tokenizer: &mut Tokenizer) -> State {
    tokenizer.check(State::Next(continue_blank), State::Next(continue_filled));
    State::Retry(tokenize_blank_line)
}

fn marker(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b':') {
        return State::Nok;
    }
    tokenizer.enter(Name::DEF_LIST_DESCRIPTION_PREFIX);
    if tokenizer.tokenize_state.def_list.pending_spread {
        tokenizer.enter(Name::DEF_LIST_DESCRIPTION_SPREAD);
        tokenizer.exit(Name::DEF_LIST_DESCRIPTION_SPREAD);
    }
    tokenizer.enter(Name::DEF_LIST_DESCRIPTION_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::DEF_LIST_DESCRIPTION_MARKER);
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
    let start = tokenizer
        .events
        .iter()
        .rposition(|event| event.name == Name::DEF_LIST_DESCRIPTION)
        .unwrap_or_default();
    // Columns, not bytes; a blank first line counts one more, as micromark's
    let mut prefix = tokenizer.point.column - tokenizer.events[start].point.column;
    if blank {
        prefix += 1;
    }
    let continued = tokenizer.tokenize_state.document_continued;
    prefix += read_def_list_indent(tokenizer, start);
    if let Some(container) = tokenizer
        .tokenize_state
        .document_container_stack
        .get_mut(continued)
    {
        container.blank_initial = blank;
        container.blank_further = false;
        container.blank_last = false;
        container.size = prefix;
    }
    tokenizer.exit(Name::DEF_LIST_DESCRIPTION_PREFIX);
    let track = &mut tokenizer.tokenize_state.def_list;
    let start = std::mem::take(&mut track.pending);
    track.starts.push(start);
    State::Ok
}

fn continue_blank(tokenizer: &mut Tokenizer) -> State {
    let continued = tokenizer.tokenize_state.document_continued;
    let Some(container) = tokenizer
        .tokenize_state
        .document_container_stack
        .get_mut(continued)
    else {
        return State::Nok;
    };
    container.blank_further = container.blank_further || container.blank_initial;
    container.blank_last = true;
    let size = container.size;
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        State::Retry(tokenize_space_or_tab(tokenizer, 0, size))
    } else {
        State::Ok
    }
}

// A description that started blank ends at the first filled line after a blank one
fn continue_filled(tokenizer: &mut Tokenizer) -> State {
    let continued = tokenizer.tokenize_state.document_continued;
    let Some(container) = tokenizer
        .tokenize_state
        .document_container_stack
        .get_mut(continued)
    else {
        return State::Nok;
    };
    let further = container.blank_further;
    container.blank_further = false;
    container.blank_initial = false;
    if further || !matches!(tokenizer.current, Some(b'\t' | b' ')) {
        return State::Nok;
    }
    container.blank_last = false;
    let size = container.size;
    State::Retry(tokenize_space_or_tab(tokenizer, size, size))
}
