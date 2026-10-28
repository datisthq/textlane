use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab;
use crate::models::construct::State;
use crate::utils::tokenizer::driver::Tokenizer;

/// A line of only spaces and tabs, up to its line ending, which it leaves.
pub(crate) fn tokenize_blank_line(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(after), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
    } else {
        State::Retry(after)
    }
}

fn after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => State::Ok,
        _ => State::Nok,
    }
}
