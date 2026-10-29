use textlane_core::hast::{Element, Node, Property, PropertyItem, PropertyValue, Text};
use textlane_core::{Allocator, ArenaVec, Error};

use crate::actions::code::tokenize::tokenize_code;
use crate::models::highlight::Themes;

/// The `<pre>` Shiki's `codeToHast` makes of code: a `span.line` per line, a styled
/// `span` per token.
pub(crate) fn highlight_code<'a>(
    code: &str,
    language: &str,
    themes: &Themes,
    alloc: &'a Allocator,
) -> Result<Node<'a>, Error> {
    let lines = tokenize_code(code, language, themes)?;
    let mut children = ArenaVec::with_capacity_in(lines.len() * 2, alloc);
    for (index, line) in lines.into_iter().enumerate() {
        if index > 0 {
            children.push(create_text("\n", alloc));
        }
        let mut spans = ArenaVec::with_capacity_in(line.len(), alloc);
        for token in line {
            let mut properties = ArenaVec::new_in(alloc);
            if !token.style.is_empty() {
                properties.push(create_property("style", &token.style, alloc));
            }
            let mut text = ArenaVec::with_capacity_in(1, alloc);
            text.push(create_text(&token.text, alloc));
            spans.push(create_element("span", properties, text));
        }
        let mut properties = ArenaVec::with_capacity_in(1, alloc);
        properties.push(create_class(&["line"], alloc));
        children.push(create_element("span", properties, spans));
    }
    let mut code = ArenaVec::with_capacity_in(1, alloc);
    code.push(create_element("code", ArenaVec::new_in(alloc), children));
    let mut properties = ArenaVec::with_capacity_in(3, alloc);
    let names = themes.list.iter().map(|theme| theme.colors.name);
    let class: Vec<&str> = match themes.single {
        true => ["shiki"].into_iter().chain(names).collect(),
        false => ["shiki", "shiki-themes"].into_iter().chain(names).collect(),
    };
    properties.push(create_class(&class, alloc));
    properties.push(create_property(
        "style",
        &stringify_root_style(themes),
        alloc,
    ));
    properties.push(Property {
        name: "tabIndex",
        value: PropertyValue::Number(0.0),
    });
    Ok(create_element("pre", properties, code))
}

// Shiki's `mapThemeColors` for the background, then the text
fn stringify_root_style(themes: &Themes) -> String {
    let mut style = String::new();
    for (property, variable) in [("background-color", "-bg"), ("color", "")] {
        for (index, theme) in themes.list.iter().enumerate() {
            let value = match variable {
                "-bg" => theme.colors.bg,
                _ => theme.colors.fg,
            };
            if !style.is_empty() {
                style.push(';');
            }
            match index {
                0 => style.push_str(&format!("{property}:{value}")),
                _ => style.push_str(&format!("--shiki-{}{variable}:{value}", theme.mode)),
            }
        }
    }
    style
}

fn create_element<'a>(
    tag_name: &'a str,
    properties: ArenaVec<'a, Property<'a>>,
    children: ArenaVec<'a, Node<'a>>,
) -> Node<'a> {
    Node::Element(Element {
        tag_name,
        properties,
        children,
        position: None,
        data: None,
        content: None,
    })
}

fn create_text<'a>(value: &str, alloc: &'a Allocator) -> Node<'a> {
    Node::Text(Text {
        value: alloc.alloc_str(value),
        position: None,
        data: None,
    })
}

fn create_property<'a>(name: &'a str, value: &str, alloc: &'a Allocator) -> Property<'a> {
    Property {
        name,
        value: PropertyValue::String(alloc.alloc_str(value)),
    }
}

fn create_class<'a>(names: &[&str], alloc: &'a Allocator) -> Property<'a> {
    let mut items = ArenaVec::with_capacity_in(names.len(), alloc);
    for name in names {
        items.push(PropertyItem::String(alloc.alloc_str(name)));
    }
    Property {
        name: "className",
        value: PropertyValue::List(items),
    }
}
