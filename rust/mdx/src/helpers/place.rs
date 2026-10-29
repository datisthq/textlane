use crate::actions::hast::convert::Snippet;
use crate::models::estree::{Id, Store};
use crate::models::name::Key;

/// unist-util-stringify-position of unist-util-position-from-estree: where a node read from
/// a snippet sits in the document, `1:1-1:31`, or nothing when that is not known.
pub(crate) fn stringify_place(store: &Store, snippets: &[Snippet], id: Id) -> String {
    let (Some(start), Some(end)) =
        (store.number(id, Key::START), store.number(id, Key::END))
    else {
        return String::new();
    };
    let (start, end) = (start as usize, end as usize);
    let Some(snippet) = snippets.iter().find(|snippet| {
        snippet.base <= start && end <= snippet.base + snippet.value.len()
    }) else {
        return String::new();
    };
    let Some(origin) = snippet.start else {
        return String::new();
    };
    let point = |offset: usize| {
        let (mut line, mut column) = (origin.line, origin.column);
        let before = snippet
            .value
            .get(..offset - snippet.base)
            .unwrap_or_default();
        let mut characters = before.chars().peekable();
        while let Some(character) = characters.next() {
            match character {
                '\r' if characters.peek() == Some(&'\n') => {}
                '\n' | '\r' => {
                    line += 1;
                    column = 1;
                }
                _ => column += character.len_utf16(),
            }
        }
        format!("{line}:{column}")
    };
    format!("{}-{}", point(start), point(end))
}
