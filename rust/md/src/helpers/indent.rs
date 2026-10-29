use crate::models::token::Name;
use crate::utils::tokenizer::driver::{Container, Tokenizer};

/// The indent a definition description took on this line right before the container
/// entered at `enter`: micromark's containers count the line prefix before them as their
/// own indent, and only a description's continuation is one.
pub(crate) fn read_def_list_indent(tokenizer: &Tokenizer, enter: usize) -> usize {
    let indented = tokenizer
        .events
        .get(enter + 1)
        .is_some_and(|event| event.name == Name::SPACE_OR_TAB);
    let parent = tokenizer
        .tokenize_state
        .document_continued
        .checked_sub(1)
        .and_then(|parent| {
            tokenizer
                .tokenize_state
                .document_container_stack
                .get(parent)
        });
    match (indented, parent, enter.checked_sub(1)) {
        (false, Some(parent), Some(exit))
            if parent.kind == Container::DefList
                && tokenizer.events[exit].name == Name::SPACE_OR_TAB =>
        {
            (tokenizer.events[exit].point.column
                - tokenizer.events[exit - 1].point.column) as usize
        }
        _ => 0,
    }
}
