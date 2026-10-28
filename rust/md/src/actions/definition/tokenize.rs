use crate::actions::destination::tokenize::tokenize_destination;
use crate::actions::label::tokenize::tokenize_label;
use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab;
use crate::actions::space_or_tab_eol::tokenize::tokenize_space_or_tab_eol;
use crate::actions::title::tokenize::tokenize_title;
use crate::helpers::identifier::normalize_identifier;
use crate::helpers::skip::{skip_opt_back, skip_to_back};
use crate::helpers::slice::span_of_exit;
use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// A definition, `[a]: b "c"`: a label, a destination and an optional title.
pub(crate) fn tokenize_definition(tokenizer: &mut Tokenizer) -> State {
    // Only another definition may come right before, if anything interrupted
    let after_definition = tokenizer.events.len().checked_sub(1).is_some_and(|last| {
        let before = skip_opt_back(
            &tokenizer.events,
            last,
            &[Name::LINE_ENDING, Name::SPACE_OR_TAB],
        );
        tokenizer.events[before].name == Name::DEFINITION
    });
    if tokenizer.interrupt && !after_definition {
        return State::Nok;
    }
    tokenizer.enter(Name::DEFINITION);
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(before), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
    } else {
        State::Retry(before)
    }
}

fn before(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'[') {
        return State::Nok;
    }
    let state = &mut tokenizer.tokenize_state;
    state.token_1 = Name::DEFINITION_LABEL;
    state.token_2 = Name::DEFINITION_LABEL_MARKER;
    state.token_3 = Name::DEFINITION_LABEL_STRING;
    tokenizer.attempt(State::Next(label_after), State::Next(label_nok));
    State::Retry(tokenize_label)
}

fn reset_tokens(tokenizer: &mut Tokenizer) {
    let state = &mut tokenizer.tokenize_state;
    state.token_1 = Name::DATA;
    state.token_2 = Name::DATA;
    state.token_3 = Name::DATA;
    state.token_4 = Name::DATA;
    state.token_5 = Name::DATA;
    state.size_b = 0;
}

fn label_after(tokenizer: &mut Tokenizer) -> State {
    reset_tokens(tokenizer);
    if tokenizer.current != Some(b':') {
        return State::Nok;
    }
    tokenizer.tokenize_state.end = skip_to_back(
        &tokenizer.events,
        tokenizer.events.len() - 1,
        &[Name::DEFINITION_LABEL_STRING],
    );
    tokenizer.enter(Name::DEFINITION_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::DEFINITION_MARKER);
    State::Next(marker_after)
}

fn label_nok(tokenizer: &mut Tokenizer) -> State {
    reset_tokens(tokenizer);
    State::Nok
}

fn marker_after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(destination_before),
        State::Next(destination_before),
    );
    State::Retry(tokenize_space_or_tab_eol(tokenizer, None, false))
}

fn destination_before(tokenizer: &mut Tokenizer) -> State {
    let state = &mut tokenizer.tokenize_state;
    state.token_1 = Name::DEFINITION_DESTINATION;
    state.token_2 = Name::DEFINITION_DESTINATION_LITERAL;
    state.token_3 = Name::DEFINITION_DESTINATION_LITERAL_MARKER;
    state.token_4 = Name::DEFINITION_DESTINATION_RAW;
    state.token_5 = Name::DEFINITION_DESTINATION_STRING;
    state.size_b = usize::MAX;
    tokenizer.attempt(
        State::Next(destination_after),
        State::Next(destination_missing),
    );
    State::Retry(tokenize_destination)
}

fn destination_after(tokenizer: &mut Tokenizer) -> State {
    reset_tokens(tokenizer);
    tokenizer.attempt(State::Next(after), State::Next(after));
    State::Retry(title_before)
}

fn destination_missing(tokenizer: &mut Tokenizer) -> State {
    reset_tokens(tokenizer);
    tokenizer.tokenize_state.end = 0;
    State::Nok
}

fn after(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(after_whitespace), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
    } else {
        State::Retry(after_whitespace)
    }
}

fn after_whitespace(tokenizer: &mut Tokenizer) -> State {
    let end = std::mem::take(&mut tokenizer.tokenize_state.end);
    if !matches!(tokenizer.current, None | Some(b'\n')) {
        return State::Nok;
    }
    tokenizer.exit(Name::DEFINITION);
    let (start, end) = span_of_exit(&tokenizer.events, end);
    let label = tokenizer
        .parse_state
        .bytes
        .get(start.index..end.index)
        .and_then(|label| std::str::from_utf8(label).ok())
        .unwrap_or_default();
    let identifier = normalize_identifier(label);
    tokenizer.tokenize_state.definitions.push(identifier);
    // A definition is not a paragraph: what follows may not continue it lazily
    tokenizer.interrupt = true;
    State::Ok
}

fn title_before(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b'\n' | b' ')) {
        tokenizer.attempt(State::Next(title_before_marker), State::Nok);
        State::Retry(tokenize_space_or_tab_eol(tokenizer, None, false))
    } else {
        State::Nok
    }
}

fn title_before_marker(tokenizer: &mut Tokenizer) -> State {
    let state = &mut tokenizer.tokenize_state;
    state.token_1 = Name::DEFINITION_TITLE;
    state.token_2 = Name::DEFINITION_TITLE_MARKER;
    state.token_3 = Name::DEFINITION_TITLE_STRING;
    tokenizer.attempt(State::Next(title_after), State::Nok);
    State::Retry(tokenize_title)
}

fn title_after(tokenizer: &mut Tokenizer) -> State {
    reset_tokens(tokenizer);
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(title_after_whitespace), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
    } else {
        State::Retry(title_after_whitespace)
    }
}

fn title_after_whitespace(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => State::Ok,
        _ => State::Nok,
    }
}
