use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab;
use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const TAB_SIZE: usize = 4;

/// A block quote: a `>` prefix on its first line.
pub(crate) fn tokenize_block_quote(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::BLOCK_QUOTE);
    State::Retry(continue_block_quote)
}

/// The `>` prefix that continues a block quote on a later line.
pub(crate) fn continue_block_quote(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(before), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 1, TAB_SIZE - 1))
    } else {
        State::Retry(before)
    }
}

fn before(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'>') {
        return State::Nok;
    }
    tokenizer.enter(Name::BLOCK_QUOTE_PREFIX);
    tokenizer.enter(Name::BLOCK_QUOTE_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::BLOCK_QUOTE_MARKER);
    State::Next(after)
}

// One space or tab after the marker belongs to the prefix
fn after(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'\t' | b' ') = tokenizer.current {
        tokenizer.enter(Name::SPACE_OR_TAB);
        tokenizer.consume();
        tokenizer.exit(Name::SPACE_OR_TAB);
    }
    tokenizer.exit(Name::BLOCK_QUOTE_PREFIX);
    State::Ok
}
