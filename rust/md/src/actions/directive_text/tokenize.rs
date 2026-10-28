use crate::actions::directive_attributes::tokenize::tokenize_directive_attributes;
use crate::actions::directive_label::tokenize::tokenize_directive_label;
use crate::actions::directive_name::tokenize::tokenize_directive_name;
use crate::models::construct::State;
use crate::models::directive::{DIRECTIVE_TEXT, DIRECTIVE_TEXT_MARKER, DirectiveKind};
use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// A text directive, `:name[label]{attributes}`, not right after another colon unless that
/// one is escaped.
pub(crate) fn tokenize_directive_text(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b':') {
        return State::Nok;
    }
    let after_escape = tokenizer.events.last().is_some_and(|event| {
        event.kind == Kind::Exit && event.name == Name::CHARACTER_ESCAPE
    });
    if tokenizer.previous == Some(b':') && !after_escape {
        return State::Nok;
    }
    tokenizer.enter(DIRECTIVE_TEXT);
    tokenizer.enter(DIRECTIVE_TEXT_MARKER);
    tokenizer.consume();
    tokenizer.exit(DIRECTIVE_TEXT_MARKER);
    tokenizer.tokenize_state.directive.kind = DirectiveKind::Text;
    tokenizer.attempt(State::Next(after_name), State::Nok);
    State::Next(tokenize_directive_name)
}

fn after_name(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b':') => State::Nok,
        Some(b'[') => {
            tokenizer.attempt(State::Next(after_label), State::Next(after_label));
            State::Retry(tokenize_directive_label)
        }
        _ => State::Retry(after_label),
    }
}

fn after_label(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'{') {
        tokenizer.attempt(State::Next(after_attributes), State::Next(after_attributes));
        return State::Retry(tokenize_directive_attributes);
    }
    State::Retry(after_attributes)
}

fn after_attributes(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(DIRECTIVE_TEXT);
    State::Ok
}
