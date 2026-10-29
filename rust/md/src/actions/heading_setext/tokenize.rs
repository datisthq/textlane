use crate::actions::space_or_tab::tokenize::{
    tokenize_space_or_tab, tokenize_space_or_tab_indent,
};
use crate::helpers::content::{find_chunks, find_paragraph_start};
use crate::helpers::skip::skip_opt_back;
use crate::models::construct::{Resolver, State};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// A setext heading's underline: `=` or `-` under a paragraph, which becomes the heading.
pub(crate) fn tokenize_heading_setext(tokenizer: &mut Tokenizer) -> State {
    let bytes = tokenizer.parse_state.bytes;
    let marker = bytes[tokenizer.point.index..]
        .iter()
        .find(|byte| !matches!(byte, b' ' | b'\t'));
    if !matches!(marker, Some(b'-' | b'=')) {
        return State::Nok;
    }
    let previous = tokenizer.events.len().checked_sub(1).map(|last| {
        skip_opt_back(
            &tokenizer.events,
            last,
            &[Name::LINE_ENDING, Name::SPACE_OR_TAB],
        )
    });
    let after_content =
        previous.filter(|previous| tokenizer.events[*previous].name == Name::CONTENT);
    if tokenizer.lazy || tokenizer.pierce || after_content.is_none() {
        return State::Nok;
    }
    tokenizer.tokenize_state.setext_content = after_content;
    tokenizer.enter(Name::HEADING_SETEXT_UNDERLINE);
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(before), State::Nok);
        State::Retry(tokenize_space_or_tab_indent(tokenizer, 0))
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
    if !matches!(tokenizer.current, None | Some(b'\n')) {
        return State::Nok;
    }
    // micromark's underline needs a paragraph above it; content holding only definitions
    // has none, yet the underline's check still closes it, so this line is new content.
    // Checked once the underline matched, which ends the content: each is read once
    if !has_paragraph(tokenizer) {
        tokenizer.tokenize_state.content_break_line = Some(tokenizer.point.line);
        return State::Nok;
    }
    tokenizer.interrupt = false;
    tokenizer.register_resolver(Resolver::HeadingSetext);
    tokenizer.exit(Name::HEADING_SETEXT_UNDERLINE);
    State::Ok
}

fn has_paragraph(tokenizer: &mut Tokenizer) -> bool {
    let Some(exit) = tokenizer.tokenize_state.setext_content.take() else {
        return false;
    };
    find_chunks(&tokenizer.events, exit, 0).is_some_and(|chunks| {
        find_paragraph_start(tokenizer.parse_state, &tokenizer.events, &chunks).is_some()
    })
}
