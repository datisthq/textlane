use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// A GFM task list check, `[ ]` or `[x]`, at the very start of a list item's first
/// paragraph and followed by whitespace and more content.
pub(crate) fn tokenize_gfm_task_list_item_check(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'[')
        || tokenizer.previous.is_some()
        || !tokenizer.tokenize_state.task_list_check_allowed
    {
        return State::Nok;
    }
    tokenizer.enter(Name::TASK_LIST_CHECK);
    tokenizer.enter(Name::TASK_LIST_CHECK_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::TASK_LIST_CHECK_MARKER);
    State::Next(inside)
}

fn inside(tokenizer: &mut Tokenizer) -> State {
    let name = match tokenizer.current {
        // GitHub files, unlike its comments, take a line ending as unchecked too
        Some(b'\t' | b'\n' | b' ') => Name::TASK_LIST_CHECK_VALUE_UNCHECKED,
        Some(b'X' | b'x') => Name::TASK_LIST_CHECK_VALUE_CHECKED,
        _ => return State::Nok,
    };
    tokenizer.enter(name);
    tokenizer.consume();
    tokenizer.exit(name);
    State::Next(close)
}

fn close(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b']') {
        return State::Nok;
    }
    tokenizer.enter(Name::TASK_LIST_CHECK_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::TASK_LIST_CHECK_MARKER);
    tokenizer.exit(Name::TASK_LIST_CHECK);
    State::Next(after)
}

fn after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => State::Ok,
        Some(b'\t' | b' ') => {
            tokenizer.check(State::Ok, State::Nok);
            State::Retry(space_then_non_space)
        }
        _ => State::Nok,
    }
}

// Something other than whitespace follows: the end of the input means nothing does
fn space_then_non_space(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(space_then_non_space)
        }
        None => State::Nok,
        Some(_) => State::Ok,
    }
}
