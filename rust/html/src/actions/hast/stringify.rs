use textlane_core::hast::{Element, Node, PropertyItem, PropertyValue};

use crate::models::options::StringifyOptions;

// Elements that have no closing tag, as html-void-elements lists them
const VOID_ELEMENTS: [&str; 20] = [
    "area", "base", "basefont", "bgsound", "br", "col", "command", "embed", "frame",
    "hr", "image", "img", "input", "keygen", "link", "meta", "param", "source", "track",
    "wbr",
];

/// Serialize hast as HTML, as hast-util-to-html does.
pub fn stringify_hast(node: &Node<'_>, options: &StringifyOptions) -> String {
    let mut output = String::new();
    write_node(&mut output, node, options);
    output
}

fn write_node(output: &mut String, node: &Node<'_>, options: &StringifyOptions) {
    match node {
        Node::Root(root) => {
            for child in &root.children {
                write_node(output, child, options);
            }
        }
        Node::Element(element) => write_element(output, element, options),
        Node::Text(text) => write_escaped(output, text.value, &['<', '&']),
        Node::Comment(comment) => {
            output.push_str("<!--");
            output.push_str(comment.value);
            output.push_str("-->");
        }
        Node::Doctype(_) => output.push_str("<!doctype html>"),
    }
}

fn write_element(output: &mut String, element: &Element<'_>, options: &StringifyOptions) {
    output.push('<');
    output.push_str(element.tag_name);
    for property in &element.properties {
        write_attribute(output, property.name, &property.value);
    }
    output.push('>');
    if VOID_ELEMENTS.contains(&element.tag_name) {
        return;
    }
    for child in &element.children {
        write_node(output, child, options);
    }
    output.push_str("</");
    output.push_str(element.tag_name);
    output.push('>');
}

fn write_attribute(output: &mut String, name: &str, value: &PropertyValue<'_>) {
    let attribute = match name {
        "className" => "class",
        "htmlFor" => "for",
        _ => name,
    };
    let value = match value {
        PropertyValue::Null | PropertyValue::Boolean(false) => return,
        PropertyValue::Boolean(true) => {
            output.push(' ');
            output.push_str(attribute);
            return;
        }
        PropertyValue::Number(number) => number.to_string(),
        PropertyValue::String(value) => (*value).to_string(),
        PropertyValue::List(items) => items
            .iter()
            .map(|item| match item {
                PropertyItem::Number(number) => number.to_string(),
                PropertyItem::String(value) => (*value).to_string(),
            })
            .collect::<Vec<_>>()
            .join(" "),
    };
    output.push(' ');
    output.push_str(attribute);
    output.push_str("=\"");
    write_escaped(output, &value, &['"', '&']);
    output.push('"');
}

// stringify-entities writes the characters it escapes as hexadecimal references
fn write_escaped(output: &mut String, value: &str, subset: &[char]) {
    for character in value.chars() {
        if subset.contains(&character) {
            output.push_str(&format!("&#x{:X};", u32::from(character)));
        } else {
            output.push(character);
        }
    }
}
