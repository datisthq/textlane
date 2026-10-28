use textlane_core::ArenaVec;
use textlane_core::hast::{self, Property, PropertyItem, PropertyValue};
use textlane_core::mdast::FootnoteReference;

use crate::helpers::uri::normalize_uri;
use crate::utils::converter::state::Converter;

/// A footnote call becomes `sup` around `a` to its footnote, numbered in call order.
pub(crate) fn convert_footnote_reference<'a>(
    converter: &mut Converter<'a, '_>,
    node: &FootnoteReference<'a>,
) -> hast::Node<'a> {
    let alloc = converter.alloc;
    let id = node.identifier.to_uppercase();
    let safe_id = normalize_uri(&id.to_lowercase());
    let next = converter.footnote_order.len();
    let (index, count) = match converter.footnote_counts.get_mut(&id) {
        Some((index, count)) => {
            *count += 1;
            (*index, *count)
        }
        None => {
            converter.footnote_order.push(id.clone());
            converter.footnote_counts.insert(id, (next, 1));
            (next, 1)
        }
    };
    let prefix = converter.clobber_prefix();
    let href = converter.concat(&["#", prefix, "fn-", &safe_id]);
    let suffix = if count > 1 {
        format!("-{count}")
    } else {
        String::new()
    };
    let id = converter.concat(&[prefix, "fnref-", &safe_id, &suffix]);
    let mut described_by = ArenaVec::with_capacity_in(1, alloc);
    described_by.push(PropertyItem::String("footnote-label"));
    let mut properties = ArenaVec::with_capacity_in(4, alloc);
    converter.push_property(&mut properties, "href", Some(href));
    converter.push_property(&mut properties, "id", Some(id));
    properties.push(Property {
        name: "dataFootnoteRef",
        value: PropertyValue::Boolean(true),
    });
    properties.push(Property {
        name: "ariaDescribedBy",
        value: PropertyValue::List(described_by),
    });
    let counter = alloc.alloc_str(&(index + 1).to_string());
    let mut text = ArenaVec::with_capacity_in(1, alloc);
    text.push(converter.text(counter));
    let link = converter.element_with("a", properties, text, node.position);
    let mut children = ArenaVec::with_capacity_in(1, alloc);
    children.push(link);
    converter.element("sup", children, node.position)
}
