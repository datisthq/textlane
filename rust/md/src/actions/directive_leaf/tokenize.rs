use crate::actions::directive_attributes::tokenize::tokenize_directive_attributes;
use crate::actions::directive_label::tokenize::tokenize_directive_label;
use crate::actions::directive_name::tokenize::tokenize_directive_name;
use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab;
use crate::models::construct::State;
use crate::models::directive::{DIRECTIVE_LEAF, DIRECTIVE_LEAF_SEQUENCE, DirectiveKind};
use crate::utils::tokenizer::driver::Tokenizer;

const TAB_SIZE: usize = 4;

/// A leaf directive, `::name[label]{attributes}`, alone on its line.
pub(crate) fn tokenize_directive_leaf(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(before), State::Nok);
        return State::Retry(tokenize_space_or_tab(tokenizer, 0, TAB_SIZE - 1));
    }
    State::Retry(before)
}

fn before(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b':') {
        return State::Nok;
    }
    tokenizer.enter(DIRECTIVE_LEAF);
    tokenizer.enter(DIRECTIVE_LEAF_SEQUENCE);
    tokenizer.consume();
    State::Next(in_start)
}

fn in_start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b':') {
        return State::Nok;
    }
    tokenizer.consume();
    tokenizer.exit(DIRECTIVE_LEAF_SEQUENCE);
    tokenizer.tokenize_state.directive.kind = DirectiveKind::Leaf;
    tokenizer.attempt(State::Next(after_name), State::Nok);
    State::Next(tokenize_directive_name)
}

fn after_name(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'[') {
        tokenizer.attempt(State::Next(after_label), State::Next(after_label));
        return State::Retry(tokenize_directive_label);
    }
    State::Retry(after_label)
}

fn after_label(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'{') {
        tokenizer.attempt(State::Next(after_attributes), State::Next(after_attributes));
        return State::Retry(tokenize_directive_attributes);
    }
    State::Retry(after_attributes)
}

fn after_attributes(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(end), State::Nok);
        return State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX));
    }
    State::Retry(end)
}

fn end(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(DIRECTIVE_LEAF);
            tokenizer.interrupt = false;
            State::Ok
        }
        _ => State::Nok,
    }
}
