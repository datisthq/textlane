use textlane_core::hast::{Element, Node, Property, PropertyValue};
use textlane_core::{Allocator, ArenaVec, find_gemoji_shortcodes, read_gemoji_label};

use crate::helpers::prose::{create_text, replace_prose};
use crate::models::options::Gemoji;

/// Turn the gemoji shortcodes in text, `:rocket:`, into their emoji; with `accessible`,
/// each in a `<span role="img" aria-label="rocket emoji">`.
pub fn replace_shortcodes<'a>(
    node: &mut Node<'a>,
    options: &Gemoji,
    alloc: &'a Allocator,
) {
    replace_prose(
        node,
        &options.ignore,
        &options.ignore_attributes,
        alloc,
        |text, out| {
            let found = find_gemoji_shortcodes(text);
            if found.is_empty() {
                return false;
            }
            let mut plain = String::new();
            let mut last = 0;
            for (start, end, emoji) in found {
                plain.push_str(&text[last..start]);
                match options.accessible {
                    true => {
                        if !plain.is_empty() {
                            out.push(create_text(&std::mem::take(&mut plain), alloc));
                        }
                        let label = read_gemoji_label(&text[start + 1..end - 1]);
                        out.push(create_label(emoji, &format!("{label} emoji"), alloc));
                    }
                    false => plain.push_str(emoji),
                }
                last = end;
            }
            plain.push_str(&text[last..]);
            if !plain.is_empty() {
                out.push(create_text(&plain, alloc));
            }
            true
        },
    );
}

fn create_label<'a>(emoji: &str, label: &str, alloc: &'a Allocator) -> Node<'a> {
    let mut properties = ArenaVec::with_capacity_in(2, alloc);
    properties.push(Property {
        name: "role",
        value: PropertyValue::String("img"),
    });
    properties.push(Property {
        name: "ariaLabel",
        value: PropertyValue::String(alloc.alloc_str(label)),
    });
    let mut children = ArenaVec::with_capacity_in(1, alloc);
    children.push(create_text(emoji, alloc));
    Node::Element(Element {
        tag_name: "span",
        properties,
        children,
        position: None,
        data: None,
        content: None,
    })
}

#[cfg(test)]
#[path = "gemoji_unit.rs"]
mod unit;
