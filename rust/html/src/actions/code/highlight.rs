use textlane_core::hast::{Element, Node, Property, PropertyItem, PropertyValue, Text};
use textlane_core::{Allocator, ArenaVec, Error};

use crate::actions::code::tokenize::{replace_color, tokenize_code};
use crate::actions::code::transform::transform_code;
use crate::models::highlight::{CodeBlock, CodeLine, Themes, add_classes};
use crate::models::options::Highlight;

/// The `<pre>` Shiki's `codeToHast` makes of code: a `span.line` per line, a styled
/// `span` per token, after the transformers `options` names.
pub(crate) fn highlight_code<'a>(
    code: &str,
    language: &str,
    meta: &str,
    themes: &Themes,
    options: &Highlight,
    alloc: &'a Allocator,
) -> Result<Node<'a>, Error> {
    let lines = tokenize_code(code, language, themes)?;
    let mut block = CodeBlock {
        lines: lines
            .into_iter()
            .map(|tokens| CodeLine {
                classes: vec!["line".to_owned()],
                tokens: tokens.into_iter().map(Some).collect(),
                removed: false,
            })
            .collect(),
        language: language.to_owned(),
        meta: meta.to_owned(),
        pre_classes: Vec::new(),
        code_classes: Vec::new(),
    };
    transform_code(&mut block, &options.transformers);
    if options.add_language_class {
        add_classes(&mut block.code_classes, &[format!("language-{language}")]);
    }
    let last = block.lines.len().saturating_sub(1);
    let mut children = ArenaVec::with_capacity_in(block.lines.len() * 2, alloc);
    // Shiki puts a line ending after each line but the last, and a removed line takes its own
    for (index, line) in block.lines.iter().enumerate() {
        if line.removed {
            continue;
        }
        let tokens = line.tokens.iter().flatten();
        let mut spans = ArenaVec::with_capacity_in(line.tokens.len(), alloc);
        for token in tokens {
            let mut properties = ArenaVec::new_in(alloc);
            if !token.style.is_empty() {
                properties.push(create_property("style", &token.style, alloc));
            }
            if !token.classes.is_empty() {
                properties.push(create_class(&token.classes, alloc));
            }
            let mut text = ArenaVec::with_capacity_in(1, alloc);
            text.push(create_text(&token.text, alloc));
            spans.push(create_element("span", properties, text));
        }
        let mut properties = ArenaVec::with_capacity_in(1, alloc);
        properties.push(create_class(&line.classes, alloc));
        children.push(create_element("span", properties, spans));
        if index < last {
            children.push(create_text("\n", alloc));
        }
    }
    let mut code_properties = ArenaVec::new_in(alloc);
    if !block.code_classes.is_empty() {
        code_properties.push(create_class(&block.code_classes, alloc));
    }
    let mut code = ArenaVec::with_capacity_in(1, alloc);
    code.push(create_element("code", code_properties, children));
    let mut properties = ArenaVec::with_capacity_in(4, alloc);
    let names = themes.list.iter().map(|theme| theme.colors.name.to_owned());
    let mut class: Vec<String> = match themes.single {
        true => ["shiki".to_owned()].into_iter().chain(names).collect(),
        false => ["shiki".to_owned(), "shiki-themes".to_owned()]
            .into_iter()
            .chain(names)
            .collect(),
    };
    add_classes(&mut class, &block.pre_classes);
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
    if let Some(name) = options
        .meta_property
        .as_deref()
        .filter(|_| !meta.is_empty())
    {
        properties.push(create_property(alloc.alloc_str(name), meta, alloc));
    }
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
            let value = replace_color(value.to_owned(), theme);
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

fn create_class<'a>(names: &[String], alloc: &'a Allocator) -> Property<'a> {
    let mut items = ArenaVec::with_capacity_in(names.len(), alloc);
    for name in names {
        items.push(PropertyItem::String(alloc.alloc_str(name)));
    }
    Property {
        name: "className",
        value: PropertyValue::List(items),
    }
}
