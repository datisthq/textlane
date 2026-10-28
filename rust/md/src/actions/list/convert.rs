use textlane_core::ArenaVec;
use textlane_core::hast::{self, Element, Property, PropertyValue};
use textlane_core::mdast::{self, List};

use crate::actions::list_item::convert::create_class_name;
use crate::utils::converter::state::Converter;

/// A list becomes `ol` (with `start` unless it is 1) or `ul`; one with task items is
/// marked as containing a task list.
pub(crate) fn convert_list<'a>(
    converter: &mut Converter<'a, '_>,
    node: &List<'a>,
    children: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    let mut properties = ArenaVec::new_in(converter.alloc);
    if let Some(start) = node.start.filter(|start| *start != 1) {
        properties.push(Property {
            name: "start",
            value: PropertyValue::Number(f64::from(start)),
        });
    }
    let tasks = node.children.iter().any(
        |child| matches!(child, mdast::Node::ListItem(item) if item.checked.is_some()),
    );
    if tasks {
        properties.push(create_class_name(converter, "contains-task-list"));
    }
    hast::Node::Element(Element {
        tag_name: if node.ordered { "ol" } else { "ul" },
        properties,
        children: converter.wrap(children, true),
        position: node.position,
        content: None,
        data: None,
    })
}
