use textlane_core::hast::{Element, Node, PropertyItem, PropertyValue};

use crate::helpers::value::{find_value, is_truthy};

/// Whether a hast node is phrasing content, as hast-util-phrasing decides.
pub(crate) fn is_phrasing(node: &Node<'_>) -> bool {
    match node {
        Node::Text(_) => true,
        Node::Element(element) => {
            is_basic_phrasing(element.tag_name)
                || is_embedded(element)
                || is_body_ok_link(element)
                || (element.tag_name == "meta" && has_property(element, "itemProp"))
        }
        _ => false,
    }
}

/// Whether an element embeds other content, as hast-util-embedded decides.
pub(crate) fn is_embedded(element: &Element<'_>) -> bool {
    matches!(
        element.tag_name,
        "audio"
            | "canvas"
            | "embed"
            | "iframe"
            | "img"
            | "math"
            | "object"
            | "picture"
            | "svg"
            | "video"
    )
}

fn is_basic_phrasing(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "a" | "abbr"
            | "area"
            | "b"
            | "bdi"
            | "bdo"
            | "br"
            | "button"
            | "cite"
            | "code"
            | "data"
            | "datalist"
            | "del"
            | "dfn"
            | "em"
            | "i"
            | "input"
            | "ins"
            | "kbd"
            | "keygen"
            | "label"
            | "map"
            | "mark"
            | "meter"
            | "noscript"
            | "output"
            | "progress"
            | "q"
            | "ruby"
            | "s"
            | "samp"
            | "script"
            | "select"
            | "small"
            | "span"
            | "strong"
            | "sub"
            | "sup"
            | "template"
            | "textarea"
            | "time"
            | "u"
            | "var"
            | "wbr"
    )
}

// hast-util-is-body-ok-link: a `link` with `itemProp`, or whose `rel` only names
// `pingback`, `prefetch` or `stylesheet`
fn is_body_ok_link(element: &Element<'_>) -> bool {
    if element.tag_name != "link" {
        return false;
    }
    if is_truthy(find_value(element, "itemProp")) {
        return true;
    }
    match find_value(element, "rel") {
        Some(PropertyValue::List(items)) if !items.is_empty() => {
            items.iter().all(|item| match item {
                PropertyItem::String(value) => {
                    matches!(*value, "pingback" | "prefetch" | "stylesheet")
                }
                PropertyItem::Number(_) => false,
            })
        }
        _ => false,
    }
}

// hast-util-has-property: set to anything but `null`, `undefined` or `false`
fn has_property(element: &Element<'_>, name: &str) -> bool {
    !matches!(
        find_value(element, name),
        None | Some(PropertyValue::Null) | Some(PropertyValue::Boolean(false))
    )
}
