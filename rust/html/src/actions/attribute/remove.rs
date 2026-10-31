use textlane_core::hast::Element;

use crate::helpers::property::find_property;
use crate::helpers::schema::HTML;

/// Remove an attribute by its HTML name; whether one was set.
pub fn remove_attribute(element: &mut Element<'_>, name: &str) -> bool {
    let found = find_property(&HTML, name);
    let before = element.properties.len();
    element
        .properties
        .retain(|property| property.name != found.property);
    element.properties.len() != before
}
