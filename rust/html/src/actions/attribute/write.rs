use textlane_core::hast::{Element, PropertyItem, PropertyValue};
use textlane_core::{Allocator, ArenaVec};

use crate::helpers::property::find_property;
use crate::helpers::schema::{
    BOOLEAN, COMMA_OR_SPACE_SEPARATED, COMMA_SEPARATED, HTML, NUMBER, OVERLOADED_BOOLEAN,
    SPACE_SEPARATED,
};
use crate::helpers::value::set_property;

/// Set an attribute by its HTML name, typed as hast types it: `class` becomes a list, a
/// boolean attribute `true`, replacing one already set.
pub fn write_attribute<'a>(
    element: &mut Element<'a>,
    name: &str,
    value: &str,
    alloc: &'a Allocator,
) {
    let found = find_property(&HTML, name);
    let kinds = found.kinds;
    let value = if kinds & (BOOLEAN | OVERLOADED_BOOLEAN) != 0
        && (value.is_empty() || value.eq_ignore_ascii_case(&found.attribute))
    {
        PropertyValue::Boolean(true)
    } else if kinds & (SPACE_SEPARATED | COMMA_SEPARATED | COMMA_OR_SPACE_SEPARATED) != 0
    {
        let comma = kinds & COMMA_SEPARATED != 0;
        let mut list = ArenaVec::new_in(alloc);
        let words: Vec<&str> = match comma {
            true => value.split(',').map(str::trim).collect(),
            false => value.split_ascii_whitespace().collect(),
        };
        for word in words.into_iter().filter(|word| !word.is_empty()) {
            list.push(PropertyItem::String(alloc.alloc_str(word)));
        }
        PropertyValue::List(list)
    } else if kinds & NUMBER != 0
        && let Ok(number) = value.trim().parse::<f64>()
    {
        PropertyValue::Number(number)
    } else {
        PropertyValue::String(alloc.alloc_str(value))
    };
    set_property(element, alloc.alloc_str(&found.property), value);
}

#[cfg(test)]
#[path = "write_unit.rs"]
mod unit;
