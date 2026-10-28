use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

/// A byte order mark at the start of the document.
pub(crate) fn tokenize_bom(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(BOM[0]) {
        tokenizer.enter(Name::BYTE_ORDER_MARK);
        State::Retry(inside)
    } else {
        State::Nok
    }
}

fn inside(tokenizer: &mut Tokenizer) -> State {
    let size = tokenizer.tokenize_state.size;
    if tokenizer.current == BOM.get(size).copied() {
        tokenizer.tokenize_state.size += 1;
        tokenizer.consume();
        if tokenizer.tokenize_state.size == BOM.len() {
            tokenizer.exit(Name::BYTE_ORDER_MARK);
            tokenizer.tokenize_state.size = 0;
            State::Ok
        } else {
            State::Next(inside)
        }
    } else {
        tokenizer.tokenize_state.size = 0;
        State::Nok
    }
}
