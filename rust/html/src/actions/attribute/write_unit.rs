use textlane_core::hast::{Element, PropertyItem, PropertyValue};
use textlane_core::{Allocator, ArenaVec};

use super::write_attribute;
use crate::actions::attribute::read::read_attribute;
use crate::actions::attribute::remove::remove_attribute;

fn create_element(alloc: &Allocator) -> Element<'_> {
    Element {
        tag_name: "pre",
        properties: ArenaVec::new_in(alloc),
        children: ArenaVec::new_in(alloc),
        position: None,
        data: None,
        content: None,
    }
}

#[test]
fn writes_as_hast_types_it() {
    let alloc = Allocator::new();
    let mut element = create_element(&alloc);
    write_attribute(&mut element, "class", "a b", &alloc);
    write_attribute(&mut element, "hidden", "", &alloc);
    write_attribute(&mut element, "data-title", "x.js", &alloc);
    write_attribute(&mut element, "data-title", "y.js", &alloc);
    let names: Vec<&str> = element.properties.iter().map(|p| p.name).collect();
    assert_eq!(names, ["className", "hidden", "dataTitle"]);
    assert!(matches!(
        &element.properties[0].value,
        PropertyValue::List(items) if matches!(items[1], PropertyItem::String("b"))
    ));
    assert_eq!(element.properties[1].value, PropertyValue::Boolean(true));
    assert_eq!(
        read_attribute(&element, "data-title").as_deref(),
        Some("y.js")
    );
}

#[test]
fn removes_by_html_name() {
    let alloc = Allocator::new();
    let mut element = create_element(&alloc);
    write_attribute(&mut element, "data-title", "x", &alloc);
    assert!(remove_attribute(&mut element, "data-title"));
    assert!(!remove_attribute(&mut element, "data-title"));
    assert!(element.properties.is_empty());
}
