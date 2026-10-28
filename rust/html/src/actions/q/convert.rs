use textlane_core::hast::Element;

use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<q>`: its content in quotes, which alternate by how deep it is nested.
pub(crate) fn convert_q<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    element: &'n Element<'a>,
) -> Converted {
    state.q_nesting += 1;
    let mut contents = state.all(&element.children);
    state.q_nesting -= 1;
    let quote = match state.options.quotes.as_deref() {
        Some(quotes) if !quotes.is_empty() => {
            quotes[state.q_nesting % quotes.len()].as_str()
        }
        _ => "\"",
    };
    let mut characters = quote.chars();
    let open = characters.next();
    let close = match characters.next() {
        Some(close) => close.to_string(),
        None => quote.to_string(),
    };
    let head = contents.first().copied();
    let tail = contents.last().copied();
    match head.map(|head| &mut state.store.items[head].kind) {
        Some(Kind::Text(text)) => text.open.extend(open),
        _ => {
            let open = state
                .alloc
                .alloc_str(&open.map(String::from).unwrap_or_default());
            contents.insert(0, state.store.add_text(open, None));
        }
    }
    match tail.map(|tail| &mut state.store.items[tail].kind) {
        Some(Kind::Text(text)) => text.close.push_str(&close),
        _ => {
            let close = state.alloc.alloc_str(&close);
            contents.push(state.store.add_text(close, None));
        }
    }
    Converted::Many(contents)
}
