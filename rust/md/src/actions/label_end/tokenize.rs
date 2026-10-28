use crate::actions::destination::tokenize::tokenize_destination;
use crate::actions::label::tokenize::tokenize_label;
use crate::actions::space_or_tab_eol::tokenize::tokenize_space_or_tab_eol;
use crate::actions::title::tokenize::tokenize_title;
use crate::helpers::skip::skip_to_back;
use crate::helpers::slice::span_of_exit;
use crate::models::construct::{Resolver, State};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::{Label, LabelKind, Tokenizer};

const RESOURCE_DESTINATION_BALANCE_MAX: usize = 32;

/// `]`: closes the last label start into a link or image when a resource, a reference or a
/// definition follows; otherwise the start becomes text.
pub(crate) fn tokenize_label_end(tokenizer: &mut Tokenizer) -> State {
    let state = &tokenizer.tokenize_state;
    if tokenizer.current != Some(b']') {
        return State::Nok;
    }
    let Some(start) = state.label_starts.last() else {
        return State::Nok;
    };
    let index = state.label_starts.len() - 1;
    let inactive = start.inactive
        || (start.kind == LabelKind::Link && index < state.label_starts_inactive_before);
    tokenizer.tokenize_state.end = tokenizer.events.len();
    if inactive {
        return State::Retry(nok);
    }
    tokenizer.enter(Name::LABEL_END);
    tokenizer.enter(Name::LABEL_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::LABEL_MARKER);
    tokenizer.exit(Name::LABEL_END);
    State::Next(after)
}

fn after(tokenizer: &mut Tokenizer) -> State {
    let state = &tokenizer.tokenize_state;
    let Some(start) = state.label_starts.last() else {
        return State::Nok;
    };
    let from = tokenizer.events[start.start.1].point.index;
    let to = tokenizer.events[state.end].point.index;
    let defined = tokenizer.parse_state.is_defined(from, to);
    let otherwise = State::Next(if defined { ok } else { nok });
    match tokenizer.current {
        Some(b'(') => {
            tokenizer.attempt(State::Next(ok), otherwise);
            State::Retry(resource_start)
        }
        Some(b'[') => {
            let collapsed = State::Next(if defined { reference_not_full } else { nok });
            tokenizer.attempt(State::Next(ok), collapsed);
            State::Retry(reference_full)
        }
        _ => State::Retry(if defined { ok } else { nok }),
    }
}

fn reference_not_full(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(ok), State::Next(nok));
    State::Retry(reference_collapsed)
}

fn ok(tokenizer: &mut Tokenizer) -> State {
    let state = &mut tokenizer.tokenize_state;
    let Some(start) = state.label_starts.pop() else {
        return State::Nok;
    };
    // Links may not contain links: every link start before this one can no longer close
    if start.kind != LabelKind::Image {
        state.label_starts_inactive_before = state.label_starts.len();
    }
    state.labels.push(Label {
        kind: start.kind,
        start: start.start,
        end: (state.end, tokenizer.events.len() - 1),
    });
    state.end = 0;
    state.gfm_footnote_image = None;
    tokenizer.register_resolver_before(Resolver::Label);
    State::Ok
}

fn nok(tokenizer: &mut Tokenizer) -> State {
    let state = &mut tokenizer.tokenize_state;
    if let Some(start) = state.label_starts.pop() {
        state.label_starts_loose.push(start);
    }
    state.label_starts_inactive_before = state
        .label_starts_inactive_before
        .min(state.label_starts.len());
    state.end = 0;
    State::Nok
}

fn resource_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::RESOURCE);
    tokenizer.enter(Name::RESOURCE_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::RESOURCE_MARKER);
    State::Next(resource_before)
}

fn resource_before(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b'\n' | b' ')) {
        tokenizer.attempt(State::Next(resource_open), State::Next(resource_open));
        State::Retry(tokenize_space_or_tab_eol(tokenizer, None, false))
    } else {
        State::Retry(resource_open)
    }
}

fn resource_open(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b')') {
        return State::Retry(resource_end);
    }
    let state = &mut tokenizer.tokenize_state;
    state.token_1 = Name::RESOURCE_DESTINATION;
    state.token_2 = Name::RESOURCE_DESTINATION_LITERAL;
    state.token_3 = Name::RESOURCE_DESTINATION_LITERAL_MARKER;
    state.token_4 = Name::RESOURCE_DESTINATION_RAW;
    state.token_5 = Name::RESOURCE_DESTINATION_STRING;
    state.size_b = RESOURCE_DESTINATION_BALANCE_MAX;
    tokenizer.attempt(
        State::Next(resource_destination_after),
        State::Next(resource_destination_missing),
    );
    State::Retry(tokenize_destination)
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

fn resource_destination_after(tokenizer: &mut Tokenizer) -> State {
    reset_tokens(tokenizer);
    if matches!(tokenizer.current, Some(b'\t' | b'\n' | b' ')) {
        tokenizer.attempt(State::Next(resource_between), State::Next(resource_end));
        State::Retry(tokenize_space_or_tab_eol(tokenizer, None, false))
    } else {
        State::Retry(resource_end)
    }
}

fn resource_destination_missing(tokenizer: &mut Tokenizer) -> State {
    reset_tokens(tokenizer);
    State::Nok
}

fn resource_between(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'"' | b'\'' | b'(') => {
            let state = &mut tokenizer.tokenize_state;
            state.token_1 = Name::RESOURCE_TITLE;
            state.token_2 = Name::RESOURCE_TITLE_MARKER;
            state.token_3 = Name::RESOURCE_TITLE_STRING;
            tokenizer.attempt(State::Next(resource_title_after), State::Nok);
            State::Retry(tokenize_title)
        }
        _ => State::Retry(resource_end),
    }
}

fn resource_title_after(tokenizer: &mut Tokenizer) -> State {
    reset_tokens(tokenizer);
    if matches!(tokenizer.current, Some(b'\t' | b'\n' | b' ')) {
        tokenizer.attempt(State::Next(resource_end), State::Next(resource_end));
        State::Retry(tokenize_space_or_tab_eol(tokenizer, None, false))
    } else {
        State::Retry(resource_end)
    }
}

fn resource_end(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b')') {
        return State::Nok;
    }
    tokenizer.enter(Name::RESOURCE_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::RESOURCE_MARKER);
    tokenizer.exit(Name::RESOURCE);
    State::Ok
}

fn reference_full(tokenizer: &mut Tokenizer) -> State {
    let state = &mut tokenizer.tokenize_state;
    state.token_1 = Name::REFERENCE;
    state.token_2 = Name::REFERENCE_MARKER;
    state.token_3 = Name::REFERENCE_STRING;
    tokenizer.attempt(
        State::Next(reference_full_after),
        State::Next(reference_full_missing),
    );
    State::Retry(tokenize_label)
}

fn reference_full_after(tokenizer: &mut Tokenizer) -> State {
    reset_tokens(tokenizer);
    let exit = skip_to_back(
        &tokenizer.events,
        tokenizer.events.len() - 1,
        &[Name::REFERENCE_STRING],
    );
    let (start, end) = span_of_exit(&tokenizer.events, exit);
    if tokenizer.parse_state.is_defined(start.index, end.index) {
        State::Ok
    } else {
        State::Nok
    }
}

fn reference_full_missing(tokenizer: &mut Tokenizer) -> State {
    reset_tokens(tokenizer);
    State::Nok
}

fn reference_collapsed(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::REFERENCE);
    tokenizer.enter(Name::REFERENCE_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::REFERENCE_MARKER);
    State::Next(reference_collapsed_open)
}

fn reference_collapsed_open(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b']') {
        return State::Nok;
    }
    tokenizer.enter(Name::REFERENCE_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::REFERENCE_MARKER);
    tokenizer.exit(Name::REFERENCE);
    State::Ok
}
