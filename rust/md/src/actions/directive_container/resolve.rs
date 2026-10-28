use crate::models::directive::{CHUNK_DOCUMENT, DIRECTIVE_CONTAINER_CONTENT};
use crate::models::event::Kind;
use crate::utils::tokenizer::driver::Tokenizer;
use crate::utils::tokenizer::subtokenize::link_to;

/// Link the content chunks of each container directive into one document.
pub(crate) fn resolve_directive_container(tokenizer: &mut Tokenizer) {
    let mut previous = None;
    for index in 0..tokenizer.events.len() {
        let event = &tokenizer.events[index];
        if event.name == DIRECTIVE_CONTAINER_CONTENT {
            previous = None;
        } else if event.kind == Kind::Enter && event.name == CHUNK_DOCUMENT {
            if let Some(previous) = previous {
                link_to(&mut tokenizer.events, previous, index);
            }
            previous = Some(index);
        }
    }
}
