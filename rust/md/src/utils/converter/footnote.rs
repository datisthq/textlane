use std::collections::HashMap;

use textlane_core::ArenaVec;
use textlane_core::hast::{self, Property, PropertyItem, PropertyValue};
use textlane_core::mdast::FootnoteDefinition;

use crate::helpers::uri::normalize_uri;
use crate::utils::converter::state::Converter;

/// The section of footnotes that were called, in call order, as mdast-util-to-hast's
/// `footer`: calls inside a footnote add theirs to the end while it is built.
pub(crate) fn create_footer<'a>(
    converter: &mut Converter<'a, '_>,
    definitions: &HashMap<String, &FootnoteDefinition<'a>>,
) -> Option<hast::Node<'a>> {
    let alloc = converter.alloc;
    let mut items = ArenaVec::new_in(alloc);
    let mut index = 0;
    while index < converter.footnote_order.len() {
        let reference_index = index;
        index += 1;
        let Some(definition) =
            definitions.get(&converter.footnote_order[reference_index])
        else {
            continue;
        };
        let mut content = ArenaVec::new_in(alloc);
        for child in definition.children.iter() {
            content.extend(converter.convert(child));
        }
        let id = definition.identifier.to_uppercase();
        let safe_id = normalize_uri(&id.to_lowercase());
        let count = converter
            .footnote_counts
            .get(&id)
            .map_or(0, |entry| entry.1);
        let back_references =
            create_back_references(converter, &safe_id, reference_index, count);
        match content.last_mut() {
            Some(hast::Node::Element(tail)) if tail.tag_name == "p" => {
                match tail.children.last_mut() {
                    Some(hast::Node::Text(text)) => {
                        text.value = converter.concat(&[text.value, " "]);
                    }
                    _ => tail.children.push(converter.text(" ")),
                }
                tail.children.extend(back_references);
            }
            _ => content.extend(back_references),
        }
        let mut properties = ArenaVec::with_capacity_in(1, alloc);
        let prefix = converter.clobber_prefix();
        let id = converter.concat(&[prefix, "fn-", &safe_id]);
        converter.push_property(&mut properties, "id", Some(id));
        let children = converter.wrap(content, true);
        items.push(converter.element_with(
            "li",
            properties,
            children,
            definition.position,
        ));
    }
    if items.is_empty() {
        return None;
    }
    let options = converter.options;
    let label = options
        .footnote_label
        .as_deref()
        .filter(|label| !label.is_empty())
        .unwrap_or("Footnotes");
    let tag_name = options
        .footnote_label_tag_name
        .as_deref()
        .filter(|name| !name.is_empty())
        .unwrap_or("h2");
    let mut class_name = ArenaVec::with_capacity_in(1, alloc);
    class_name.push(PropertyItem::String("sr-only"));
    let mut properties = ArenaVec::with_capacity_in(2, alloc);
    properties.push(Property {
        name: "className",
        value: PropertyValue::List(class_name),
    });
    converter.push_property(&mut properties, "id", Some("footnote-label"));
    let mut text = ArenaVec::with_capacity_in(1, alloc);
    text.push(converter.text(alloc.alloc_str(label)));
    let heading =
        converter.element_with(alloc.alloc_str(tag_name), properties, text, None);
    let mut children = ArenaVec::with_capacity_in(4, alloc);
    children.push(heading);
    children.push(converter.text("\n"));
    children.push(converter.element("ol", converter.wrap(items, true), None));
    children.push(converter.text("\n"));
    let mut class_name = ArenaVec::with_capacity_in(1, alloc);
    class_name.push(PropertyItem::String("footnotes"));
    let mut properties = ArenaVec::with_capacity_in(2, alloc);
    properties.push(Property {
        name: "dataFootnotes",
        value: PropertyValue::Boolean(true),
    });
    properties.push(Property {
        name: "className",
        value: PropertyValue::List(class_name),
    });
    Some(converter.element_with("section", properties, children, None))
}

// One `a` back to each call of a footnote, separated by spaces
fn create_back_references<'a>(
    converter: &Converter<'a, '_>,
    safe_id: &str,
    reference_index: usize,
    count: usize,
) -> ArenaVec<'a, hast::Node<'a>> {
    let alloc = converter.alloc;
    let options = converter.options;
    let prefix = converter.clobber_prefix();
    let mut nodes = ArenaVec::with_capacity_in(count * 2, alloc);
    for rereference_index in 1..=count {
        if rereference_index > 1 {
            nodes.push(converter.text(" "));
        }
        let suffix = if rereference_index > 1 {
            format!("-{rereference_index}")
        } else {
            String::new()
        };
        let href = converter.concat(&["#", prefix, "fnref-", safe_id, &suffix]);
        let label = match options.footnote_back_label.as_deref() {
            Some(label) if !label.is_empty() => alloc.alloc_str(label),
            _ => alloc.alloc_str(&format!(
                "Back to reference {}{suffix}",
                reference_index + 1
            )),
        };
        let mut children = ArenaVec::with_capacity_in(2, alloc);
        match options.footnote_back_content.as_deref() {
            Some(content) if !content.is_empty() => {
                children.push(converter.text(alloc.alloc_str(content)))
            }
            _ => {
                children.push(converter.text("↩"));
                if rereference_index > 1 {
                    let mut number = ArenaVec::with_capacity_in(1, alloc);
                    let value = alloc.alloc_str(&rereference_index.to_string());
                    number.push(converter.text(value));
                    children.push(converter.element("sup", number, None));
                }
            }
        }
        let mut class_name = ArenaVec::with_capacity_in(1, alloc);
        class_name.push(PropertyItem::String("data-footnote-backref"));
        let mut properties = ArenaVec::with_capacity_in(4, alloc);
        converter.push_property(&mut properties, "href", Some(href));
        converter.push_property(&mut properties, "dataFootnoteBackref", Some(""));
        converter.push_property(&mut properties, "ariaLabel", Some(label));
        properties.push(Property {
            name: "className",
            value: PropertyValue::List(class_name),
        });
        nodes.push(converter.element_with("a", properties, children, None));
    }
    nodes
}

/// Add the footer to the converted tree, after a line ending, as `toHast` does.
pub(crate) fn append_footer<'a>(tree: &mut hast::Node<'a>, footer: hast::Node<'a>) {
    let children = match tree {
        hast::Node::Root(root) => &mut root.children,
        hast::Node::Element(element) => &mut element.children,
        _ => return,
    };
    children.push(hast::Node::Text(hast::Text {
        value: "\n",
        position: None,
        data: None,
    }));
    children.push(footer);
}
