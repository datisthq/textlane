use std::borrow::Cow;

use textlane_core::hast::Element;

use crate::helpers::selected::{OptionValues, find_selected_options, read_shown_count};
use crate::helpers::value::{find_string, find_value, is_truthy, read_value};
use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<input>`: a checkbox's mark, an image, links for emails and URLs, or the text of its
/// value, placeholder or the selected options of its `list`.
pub(crate) fn convert_input<'a>(
    state: &mut Converter<'a, '_, '_>,
    element: &Element<'a>,
) -> Converted {
    let kind = find_string(element, "type");
    let mut value = read_value(element, "value");
    if value.is_empty() {
        value = read_value(element, "placeholder");
    }
    if is_truthy(find_value(element, "disabled"))
        || matches!(kind, Some("hidden" | "file"))
    {
        return Converted::Nothing;
    }
    if matches!(kind, Some("checkbox" | "radio")) {
        let mark = if is_truthy(find_value(element, "checked")) {
            state
                .options
                .checked
                .as_deref()
                .filter(|mark| !mark.is_empty())
                .unwrap_or("[x]")
        } else {
            state
                .options
                .unchecked
                .as_deref()
                .filter(|mark| !mark.is_empty())
                .unwrap_or("[ ]")
        };
        let mark = state.alloc.alloc_str(mark);
        return Converted::One(state.store.add_text(mark, element.position));
    }
    if kind == Some("image") {
        let alt = match find_value(element, "alt") {
            Some(alt) if is_truthy(Some(alt)) => read_value(element, "alt"),
            _ => value,
        };
        if alt.is_empty() {
            return Converted::Nothing;
        }
        let source = read_value(element, "src");
        let url = state.resolve((!source.is_empty()).then_some(&*source));
        let title = read_value(element, "title");
        let title = (!title.is_empty()).then(|| state.intern(title));
        let alt = state.intern(alt);
        let image = Kind::Image { url, title, alt };
        return Converted::One(state.store.add(image, vec![], element.position));
    }
    let mut values: OptionValues = vec![];
    if !value.is_empty() {
        values.push((value.into_owned(), None));
    } else if !matches!(
        kind,
        Some("button" | "file" | "password" | "reset" | "submit")
    ) && is_truthy(find_value(element, "list"))
    {
        let list = read_value(element, "list");
        if let Some(datalist) = state.element_by_id.get(list.as_ref())
            && datalist.tag_name == "datalist"
        {
            let key = (
                *datalist as *const Element<'_> as usize,
                read_shown_count(element),
            );
            values = match state.datalists.get(&key) {
                Some(values) => values.clone(),
                None => {
                    let found = find_selected_options(datalist, element);
                    state.datalists.insert(key, found.clone());
                    found
                }
            };
        }
    }
    if values.is_empty() {
        return Converted::Nothing;
    }
    if kind == Some("password") {
        let length = values[0].0.encode_utf16().count();
        values[0] = ("•".repeat(length), None);
    }
    if matches!(kind, Some("email" | "url")) {
        let mut results = vec![];
        let last = values.len() - 1;
        for (index, (value, label)) in values.into_iter().enumerate() {
            let resolved = state.resolve(Some(&value));
            let url = if kind == Some("email") {
                state.alloc.alloc_str(&format!("mailto:{resolved}"))
            } else {
                resolved
            };
            let text = match label {
                Some(label) => state.alloc.alloc_str(&label),
                None => resolved,
            };
            let text = state.store.add_text(text, None);
            results.push(state.store.add(
                Kind::Link { url, title: None },
                vec![text],
                None,
            ));
            if index != last {
                results.push(state.store.add_text(", ", None));
            }
        }
        return Converted::Many(results);
    }
    let texts: Vec<Cow<'_, str>> = values
        .iter()
        .map(|(value, label)| match label {
            Some(label) => Cow::Owned(format!("{label} ({value})")),
            None => Cow::Borrowed(value.as_str()),
        })
        .collect();
    let text = state.alloc.alloc_str(&texts.join(", "));
    Converted::One(state.store.add_text(text, element.position))
}
