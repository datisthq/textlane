use std::sync::LazyLock;

use regex::Regex;
use textlane_core::hast::{Element, Node, PropertyItem, PropertyValue};
use textlane_core::{Allocator, Data, Error};

use crate::actions::code::highlight::highlight_code;
use crate::actions::code::tokenize::is_plain_language;
use crate::actions::theme::resolve::resolve_themes;
use crate::helpers::registry::{INJECTION_TARGETS, load_registry};
use crate::models::highlight::Themes;
use crate::models::options::{Highlight, InlineCode};

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
            let found = match child {
                Node::Element(element) => read_block(element, options)
                    .map(|block| (block, false))
                    .or_else(|| read_inline(element, options).map(|block| (block, true))),
                _ => None,
            };
            let Some(((code, language, meta), inline)) = found else {
                stack.push(child);
                continue;
            };
            if themes.is_none() {
                themes = Some(resolve_themes(options)?);
            }
            if let Some(themes) = &themes {
                let mut highlighted =
                    highlight_code(&code, &language, &meta, themes, options, alloc)?;
                // @shikijs/rehype renames the `<pre>` of inline code a `<span>`
                if let (true, Node::Element(element)) = (inline, &mut highlighted) {
                    element.tag_name = "span";
                }
                *child = highlighted;
            }
        }
    }
    Ok(())
}

// Inline code ending in `{:lang}`, as `tailing-curly-colon` reads it: the code before, and
// the language it names, never one taken by default
fn read_inline(code: &Element, options: &Highlight) -> Option<(String, String, String)> {
    if code.tag_name != "code" || options.inline != Some(InlineCode::TailingCurlyColon) {
        return None;
    }
    let mut text = String::new();
    collect_text(&code.children, &mut text);
    let captures = TAILING_CURLY_COLON.as_ref()?.captures(&text)?;
    let named = captures.get(2)?.as_str();
    let language = match is_known_language(named) {
        true => named.to_owned(),
        false => options.fallback_language.clone()?,
    };
    Some((
        captures.get(1)?.as_str().to_owned(),
        language,
        String::new(),
    ))
}

static TAILING_CURLY_COLON: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"^(.+)\{:([A-Za-z0-9_-]+)\}$").ok());

// A block's code, without one final line ending, the language to color it in and its meta
fn read_block(pre: &Element, options: &Highlight) -> Option<(String, String, String)> {
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
    Some((text, language, read_meta(code)))
}

// What follows a fence's language: `data.meta` as mdast-util-to-hast keeps it, else a
// `metastring` property
fn read_meta(code: &Element) -> String {
    if let Some(Data::Object(entries)) = code.data
        && let Some((_, Data::String(meta))) =
            entries.iter().find(|(key, _)| *key == "meta")
    {
        return (*meta).to_owned();
    }
    let property = code
        .properties
        .iter()
        .find(|property| property.name == "metastring");
    match property.map(|property| &property.value) {
        Some(PropertyValue::String(meta)) => (*meta).to_owned(),
        _ => String::new(),
    }
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
        || language == "ansi"
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
