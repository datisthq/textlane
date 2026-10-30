use textlane_core::hast::{Element, Node, PropertyValue};
use textlane_core::{Allocator, ArenaVec};

/// Put the content of each paragraph that holds only images, links around them and white
/// space in its place, as rehype-unwrap-images does.
pub fn unwrap_images<'a>(node: &mut Node<'a>, alloc: &'a Allocator) {
    let mut stack: Vec<&mut Node<'a>> = vec![node];
    while let Some(node) = stack.pop() {
        let children = match node {
            Node::Root(root) => &mut root.children,
            Node::Element(element) => &mut element.children,
            _ => continue,
        };
        let nodes = std::mem::replace(children, ArenaVec::new_in(alloc));
        // upstream revisits the spliced content, so a paragraph met there unwraps too
        let mut pending: Vec<Node<'a>> = nodes.into_iter().rev().collect();
        while let Some(child) = pending.pop() {
            match child {
                Node::Element(element)
                    if element.tag_name == "p"
                        && read_content(&element, false) == Content::Image =>
                {
                    pending.extend(element.children.into_iter().rev());
                }
                child => children.push(child),
            }
        }
        for child in children.iter_mut() {
            stack.push(child);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Content {
    Unknown,
    Image,
    Other,
}

// rehype-unwrap-images' `applicable`: images and white space, through one level of
// interactive elements such as links
fn read_content(element: &Element<'_>, in_link: bool) -> Content {
    let mut content = Content::Unknown;
    for child in element.children.iter() {
        match child {
            Node::Text(text) if is_whitespace(text.value) => {}
            Node::Element(child) if child.tag_name == "img" => content = Content::Image,
            Node::Element(child) if !in_link && is_interactive(child) => {
                match read_content(child, true) {
                    Content::Other => return Content::Other,
                    Content::Image => content = Content::Image,
                    Content::Unknown => {}
                }
            }
            _ => return Content::Other,
        }
    }
    content
}

fn is_whitespace(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\x0C' | b'\r'))
}

// hast-util-interactive
fn is_interactive(element: &Element<'_>) -> bool {
    let name = element.tag_name;
    (name == "a" && has_property(element, "href"))
        || (matches!(name, "audio" | "video") && has_property(element, "controls"))
        || (matches!(name, "object" | "img") && has_property(element, "useMap"))
        || (name == "input" && !is_hidden_input(element))
        || has_property(element, "tabIndex")
        || matches!(
            name,
            "button"
                | "details"
                | "embed"
                | "iframe"
                | "keygen"
                | "label"
                | "select"
                | "textarea"
        )
}

// hast-util-has-property: set to anything but `null`, `undefined` or `false`
fn has_property(element: &Element<'_>, name: &str) -> bool {
    element.properties.iter().any(|property| {
        property.name == name
            && !matches!(
                property.value,
                PropertyValue::Null | PropertyValue::Boolean(false)
            )
    })
}

fn is_hidden_input(element: &Element<'_>) -> bool {
    element.properties.iter().any(|property| {
        property.name == "type"
            && matches!(property.value, PropertyValue::String("hidden"))
    })
}
