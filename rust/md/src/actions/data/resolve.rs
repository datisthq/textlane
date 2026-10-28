use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// Merge adjacent data tokens into one.
pub(crate) fn resolve_data(tokenizer: &mut Tokenizer) {
    let mut index = 0;
    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];
        if event.kind == Kind::Enter && event.name == Name::DATA {
            index += 1;
            let mut exit_index = index;
            while exit_index + 1 < tokenizer.events.len()
                && tokenizer.events[exit_index + 1].name == Name::DATA
            {
                exit_index += 2;
            }
            if exit_index > index {
                tokenizer.map.add(index, exit_index - index, vec![]);
                tokenizer.events[index].point =
                    tokenizer.events[exit_index].point.clone();
                index = exit_index;
            }
        }
        index += 1;
    }
    tokenizer.map.consume(&mut tokenizer.events);
}
