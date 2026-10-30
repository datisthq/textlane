use textlane_core::hast::{Element, Node, PropertyItem, PropertyValue};
use textlane_core::{Allocator, Error};

use crate::actions::code::highlight::highlight_code;
use crate::actions::code::tokenize::is_plain_language;
use crate::actions::theme::resolve::resolve_themes;
use crate::helpers::registry::{INJECTION_TARGETS, load_registry};
use crate::models::highlight::Themes;
use crate::models::options::Highlight;

/// Color each `<pre>` whose first child is a `<code class="language-…">` as
/// @shikijs/rehype does, replacing the block with Shiki's.
pub fn highlight_hast<'a>(
    node: &mut Node<'a>,
    options: &Highlight,
    alloc: &'a Allocator,
) -> Result<(), Error> {
    // Resolved at the first block, as Shiki only fails on themes when it highlights
    let mut themes: Option<Themes> = None;
    let mut stack: Vec<&mut Node<'a>> = vec![node];
    while let Some(node) = stack.pop() {
        let children = match node {
            Node::Root(root) => &mut root.children,
            Node::Element(element) => &mut element.children,
            _ => continue,
        };
        for child in children.iter_mut() {
            if let Node::Element(element) = child
                && let Some((code, language)) = read_block(element, options)
            {
                if themes.is_none() {
                    themes = Some(resolve_themes(options)?);
                }
                if let Some(themes) = &themes {
                    *child = highlight_code(&code, &language, themes, alloc)?;
                }
                continue;
            }
            stack.push(child);
        }
    }
    Ok(())
}

// A block's code, without one final line ending, and the language to color it in
fn read_block(pre: &Element, options: &Highlight) -> Option<(String, String)> {
    if pre.tag_name != "pre" {
        return None;
    }
    let Some(Node::Element(code)) = pre.children.first() else {
        return None;
    };
    if code.tag_name != "code" {
        return None;
    }
    let named = read_language(code).filter(|language| !language.is_empty());
    let language = match named {
        None => options.default_language.clone()?,
        Some(language) if is_known_language(language) => language.to_owned(),
        Some(_) => options.fallback_language.clone()?,
    };
    let mut text = String::new();
    collect_text(&code.children, &mut text);
    if text.ends_with('\n') {
        text.pop();
    }
    Some((text, language))
}

fn read_language<'a>(code: &Element<'a>) -> Option<&'a str> {
    let property = code
        .properties
        .iter()
        .find(|property| property.name == "className")?;
    let PropertyValue::List(items) = &property.value else {
        return None;
    };
    items.iter().find_map(|item| match item {
        PropertyItem::String(name) => name.strip_prefix("language-"),
        PropertyItem::Number(_) => None,
    })
}

// Shiki names its languages and aliases in lowercase and matches them exactly
fn is_known_language(language: &str) -> bool {
    is_plain_language(language)
        || (!language.chars().any(char::is_uppercase)
            && !INJECTION_TARGETS
                .iter()
                .any(|(scope, _)| *scope == language)
            && load_registry()
                .is_some_and(|registry| registry.contains_grammar(language)))
}

// hast-util-to-string: the text of every descendant, in order
fn collect_text(children: &[Node], text: &mut String) {
    let mut stack: Vec<std::slice::Iter<Node>> = vec![children.iter()];
    while let Some(iter) = stack.last_mut() {
        match iter.next() {
            Some(Node::Text(node)) => text.push_str(node.value),
            Some(Node::Element(element)) => stack.push(element.children.iter()),
            Some(Node::Root(root)) => stack.push(root.children.iter()),
            Some(_) => {}
            None => {
                stack.pop();
            }
        }
    }
}

#[cfg(test)]
#[path = "highlight_unit.rs"]
mod unit;
