use std::collections::VecDeque;

use textlane_core::Point;

use crate::helpers::classify::{Class, classify_character};
use crate::utils::parser::store::{Kind, Place, Span, Store};

/// Split `value` into words, white space, punctuation and symbols, as parse-latin's
/// `tokenize` does, spanning bytes from `offset` on.
pub(crate) fn tokenize_value<'a>(
    store: &mut Store<'a>,
    value: &'a str,
    offset: usize,
) -> VecDeque<usize> {
    let mut children = VecDeque::new();
    let mut from = 0;
    let mut previous: Option<(Class, Option<char>)> = None;
    for (index, character) in value.char_indices() {
        let class = classify_character(character);
        // parse-latin reads UTF-16 units, none of which an astral character's halves match
        let astral = character.len_utf16() == 2;
        let class = if astral { Class::Symbol } else { class };
        if let Some((previous_class, previous_character)) = previous {
            let continues = previous_class == class
                && (matches!(class, Class::Word | Class::WhiteSpace)
                    || astral
                    || previous_character == Some(character));
            if !continues {
                children.push_back(add_token(
                    store,
                    previous_class,
                    value,
                    from,
                    index,
                    offset,
                ));
                from = index;
            }
        }
        previous = Some((class, (!astral).then_some(character)));
    }
    if let Some((class, _)) = previous {
        children.push_back(add_token(store, class, value, from, value.len(), offset));
    }
    children
}

fn add_token<'a>(
    store: &mut Store<'a>,
    class: Class,
    value: &'a str,
    from: usize,
    to: usize,
    offset: usize,
) -> usize {
    let position = Some(Span {
        start: Some(Place::Byte(Point::new(offset + from, 0))),
        end: Some(Place::Byte(Point::new(offset + to, 0))),
    });
    let slice = &value[from..to];
    match class {
        Class::Word => {
            let text = store.add(Kind::Text, slice, position);
            store.add_parent(Kind::Word, VecDeque::from([text]), position)
        }
        Class::WhiteSpace => store.add(Kind::WhiteSpace, slice, position),
        Class::Punctuation => store.add(Kind::Punctuation, slice, position),
        Class::Symbol => store.add(Kind::Symbol, slice, position),
    }
}
