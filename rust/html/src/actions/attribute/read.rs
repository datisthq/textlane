use std::borrow::Cow;

use textlane_core::hast::{Element, PropertyItem, PropertyValue};

use crate::actions::hast::stringify::write_js_number;
use crate::helpers::property::find_property;
use crate::helpers::schema::{HTML, SPACE_SEPARATED};
use crate::helpers::value::{find_value, write_value};

/// An attribute's value by its HTML name (`class`, `data-title`), as the HTML would spell
/// it: `""` for a set boolean, `None` when unset, `false` or `null`.
pub fn read_attribute<'a>(element: &Element<'a>, name: &str) -> Option<Cow<'a, str>> {
    let found = find_property(&HTML, name);
    match find_value(element, &found.property)? {
        PropertyValue::Null | PropertyValue::Boolean(false) => None,
        PropertyValue::Boolean(true) => Some(Cow::Borrowed("")),
        PropertyValue::List(items) if found.kinds & SPACE_SEPARATED != 0 => {
            let words: Vec<Cow<str>> = (items.iter())
                .map(|item| match item {
                    PropertyItem::String(word) => Cow::Borrowed(*word),
                    PropertyItem::Number(number) => Cow::Owned(write_js_number(*number)),
                })
                .collect();
            Some(Cow::Owned(words.join(" ")))
        }
        value => Some(write_value(value)),
    }
}

#[cfg(test)]
#[path = "read_unit.rs"]
mod unit;
