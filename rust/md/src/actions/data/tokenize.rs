use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// Data: everything up to a marker another construct may start at, and line endings.
pub(crate) fn tokenize_data(tokenizer: &mut Tokenizer) -> State {
    if let Some(byte) = tokenizer.current
        && tokenizer.tokenize_state.markers.contains(&byte)
    {
        tokenizer.enter(Name::DATA);
        tokenizer.consume();
        return State::Next(inside);
    }
    State::Retry(at_break)
}

fn at_break(tokenizer: &mut Tokenizer) -> State {
    if let Some(byte) = tokenizer.current
        && !tokenizer.tokenize_state.markers.contains(&byte)
    {
        if byte == b'\n' {
            tokenizer.enter(Name::LINE_ENDING);
            tokenizer.consume();
            tokenizer.exit(Name::LINE_ENDING);
            return State::Next(at_break);
        }
        tokenizer.enter(Name::DATA);
        return State::Retry(inside);
    }
    State::Ok
}

fn inside(tokenizer: &mut Tokenizer) -> State {
    if let Some(byte) = tokenizer.current
        && byte != b'\n'
        && !tokenizer.tokenize_state.markers.contains(&byte)
    {
        tokenizer.consume();
        return State::Next(inside);
    }
    tokenizer.exit(Name::DATA);
    State::Retry(at_break)
}
