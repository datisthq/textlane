use textlane_core::ArenaVec;
use textlane_core::hast;
use textlane_core::mdast::{LinkReference, ReferenceType};

use crate::helpers::uri::normalize_uri;
use crate::utils::converter::state::Converter;

/// A link reference becomes `a` to its definition, or its source text when there is none.
pub(crate) fn convert_link_reference<'a>(
    converter: &mut Converter<'a, '_>,
    node: &LinkReference<'a>,
    mut children: ArenaVec<'a, hast::Node<'a>>,
    results: &mut Vec<hast::Node<'a>>,
) {
    let Some((url, title)) = converter.find_definition(node.identifier) else {
        // As mdast-util-to-hast's `revert`: the brackets become text around the children
        let suffix = reference_suffix(node.reference_type, node.label, node.identifier);
        let head = match children.first_mut() {
            Some(hast::Node::Text(text)) => Some(text),
            _ => None,
        };
        match head {
            Some(text) => text.value = converter.concat(&["[", text.value]),
            None => children.insert(0, converter.text("[")),
        }
        let tail = match children.last_mut() {
            Some(hast::Node::Text(text)) => Some(text),
            _ => None,
        };
        match tail {
            Some(text) => text.value = converter.concat(&[text.value, &suffix]),
            None => {
                let suffix = converter.alloc.alloc_str(&suffix);
                children.push(converter.text(suffix));
            }
        }
        results.extend(children);
        return;
    };
    let href = converter.alloc.alloc_str(&normalize_uri(url));
    let mut properties = ArenaVec::new_in(converter.alloc);
    converter.push_property(&mut properties, "href", Some(href));
    converter.push_property(&mut properties, "title", title);
    results.push(converter.element_with("a", properties, children, node.position));
}

/// What closes a reference written as source text: `]`, `][]` or `][label]`.
pub(crate) fn reference_suffix(
    reference_type: ReferenceType,
    label: Option<&str>,
    identifier: &str,
) -> String {
    match reference_type {
        ReferenceType::Shortcut => "]".to_string(),
        ReferenceType::Collapsed => "][]".to_string(),
        ReferenceType::Full => {
            let label = label
                .filter(|label| !label.is_empty())
                .unwrap_or(identifier);
            format!("][{label}]")
        }
    }
}
