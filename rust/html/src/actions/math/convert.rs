use textlane_core::hast::{Element, Node};

use crate::actions::attribute::read::read_attribute;
use crate::helpers::text::read_text;
use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<math>` with `toMarkdown`'s `math`: math of its TeX annotation, display math for
/// `display="block"`; nothing without an annotation.
pub(crate) fn convert_math<'a>(
    state: &mut Converter<'a, '_, '_>,
    element: &Element<'a>,
) -> Converted {
    let Some(tex) = find_annotation(element) else {
        return Converted::Nothing;
    };
    let value = state.alloc.alloc_str(read_text(tex).trim());
    let kind = match read_attribute(element, "display").as_deref() {
        Some("block") => Kind::Math(value),
        _ => Kind::InlineMath(value),
    };
    Converted::One(state.store.add(kind, vec![], element.position))
}

fn find_annotation<'n, 'a>(element: &'n Element<'a>) -> Option<&'n Element<'a>> {
    let mut stack: Vec<&'n Element<'a>> = vec![element];
    while let Some(element) = stack.pop() {
        let tex = element.tag_name == "annotation"
            && read_attribute(element, "encoding").as_deref()
                == Some("application/x-tex");
        if tex {
            return Some(element);
        }
        stack.extend(
            element
                .children
                .iter()
                .rev()
                .filter_map(|child| match child {
                    Node::Element(child) => Some(child),
                    _ => None,
                }),
        );
    }
    None
}
