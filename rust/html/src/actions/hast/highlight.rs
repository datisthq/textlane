use std::sync::LazyLock;

use regex::Regex;
use textlane_core::hast::{Element, Node, PropertyItem, PropertyValue};
use textlane_core::{Allocator, ArenaVec, Data, Error};

use crate::actions::attribute::read::read_attribute;
use crate::actions::code::highlight::highlight_code;
use crate::actions::code::tokenize::is_plain_language;
use crate::actions::code::transform::read_classes;
use crate::actions::theme::resolve::resolve_themes;
use crate::helpers::registry::{INJECTION_TARGETS, load_registry};
use crate::models::highlight::{BlockMarks, Themes, add_classes};
use crate::models::options::{Highlight, InlineCode, LineMark};

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
        let in_pre =
            matches!(&*node, Node::Element(element) if element.tag_name == "pre");
        let children = match node {
            Node::Root(root) => &mut root.children,
            Node::Element(element) => &mut element.children,
            _ => continue,
        };
        for child in children.iter_mut() {
            let found = match child {
                Node::Element(element) => read_block(element, options)
                    .map(|block| (block, false))
                    .or_else(|| {
                        let inline = read_inline(element, options, in_pre);
                        inline.map(|block| (block, true))
                    }),
                _ => None,
            };
            let Some(((code, language, meta), inline)) = found else {
                stack.push(child);
                continue;
            };
            let marks = match (inline, &*child) {
                (false, Node::Element(pre)) => read_marks(pre, options)?,
                _ => BlockMarks::default(),
            };
            if themes.is_none() {
                themes = Some(resolve_themes(options)?);
            }
            if let Some(themes) = &themes {
                let mut highlighted = highlight_code(
                    &code, &language, &meta, &marks, themes, options, alloc,
                )?;
                if options.keep_properties
                    && let (Node::Element(old), Node::Element(new)) =
                        (&mut *child, &mut highlighted)
                {
                    keep_properties(old, new, inline);
                }
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
fn read_inline(
    code: &Element,
    options: &Highlight,
    in_pre: bool,
) -> Option<(String, String, String)> {
    if code.tag_name != "code" || in_pre {
        return None;
    }
    if options.skip_elements && has_elements(&code.children) {
        return None;
    }
    let mut text = String::new();
    collect_text(&code.children, &mut text);
    if options.inline == Some(InlineCode::Class) {
        let named = read_language(code).filter(|language| !language.is_empty())?;
        let language = match is_known_language(named) {
            true => named.to_owned(),
            false => options.fallback_language.clone()?,
        };
        return Some((text, language, String::new()));
    }
    if options.inline != Some(InlineCode::TailingCurlyColon) {
        return None;
    }
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
    if options.skip_elements && has_elements(&code.children) {
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

fn has_elements(children: &[Node]) -> bool {
    children
        .iter()
        .any(|child| matches!(child, Node::Element(_)))
}

// A block's `lineAttributes`, `wordAttribute` and `lineNumberAttribute`, read from its
// `<code>`, else its `<pre>`
fn read_marks(pre: &Element, options: &Highlight) -> Result<BlockMarks, Error> {
    let code = match pre.children.first() {
        Some(Node::Element(code)) => Some(code),
        _ => None,
    };
    let read = |name: &str| {
        code.and_then(|code| read_attribute(code, name))
            .or_else(|| read_attribute(pre, name))
    };
    let mut marks = BlockMarks::default();
    for (name, mark) in &options.line_attributes {
        let Some(value) = read(name) else {
            continue;
        };
        let ranges = read_ranges(&value).ok_or_else(|| {
            Error::new(
                "invalid-lines",
                format!("`{name}` must list lines as `2,4-5`, got `{value}`"),
            )
        })?;
        let (line, pre) = match mark {
            LineMark::Line(line) => (line, None),
            LineMark::Both { line, pre } => (line, pre.as_ref()),
        };
        marks
            .lines
            .push((ranges, read_classes(&Some(line.clone()), "")));
        if let Some(pre) = pre {
            add_classes(&mut marks.pre, &read_classes(&Some(pre.clone()), ""));
        }
    }
    if let Some(name) = &options.word_attribute
        && let Some(value) = read(name)
    {
        marks.words = value
            .split(',')
            .map(|word| word.trim().to_owned())
            .collect();
    }
    if let Some(name) = &options.line_number_attribute
        && let Some(value) = read(name)
    {
        let value = value.trim();
        marks.numbers = Some(match value.is_empty() {
            true => 1,
            false => value.parse().map_err(|_| {
                Error::new(
                    "invalid-lines",
                    format!("`{name}` must be a line number or empty, got `{value}`"),
                )
            })?,
        });
    }
    Ok(marks)
}

// `2,4-5` as inclusive ranges, `None` for anything else
fn read_ranges(value: &str) -> Option<Vec<(usize, usize)>> {
    let mut ranges = vec![];
    for part in value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let (start, end) = part.split_once('-').unwrap_or((part, part));
        let (start, end) = (start.trim().parse().ok()?, end.trim().parse().ok()?);
        if start == 0 || end < start {
            return None;
        }
        ranges.push((start, end));
    }
    Some(ranges)
}

// The source block's properties onto the one Shiki made: classes merged, Shiki's others
// winning; inline code keeps its `<code>`'s on the `<span>` that replaces it
fn keep_properties<'a>(old: &mut Element<'a>, new: &mut Element<'a>, inline: bool) {
    merge_properties(old, new);
    if inline {
        return;
    }
    let old_code = old.children.iter_mut().find_map(|child| match child {
        Node::Element(code) if code.tag_name == "code" => Some(code),
        _ => None,
    });
    let new_code = new.children.iter_mut().find_map(|child| match child {
        Node::Element(code) if code.tag_name == "code" => Some(code),
        _ => None,
    });
    if let (Some(old_code), Some(new_code)) = (old_code, new_code) {
        merge_properties(old_code, new_code);
    }
}

fn merge_properties<'a>(old: &mut Element<'a>, new: &mut Element<'a>) {
    let bump = old.properties.bump();
    for property in std::mem::replace(&mut old.properties, ArenaVec::new_in(bump)) {
        let existing = new
            .properties
            .iter_mut()
            .find(|own| own.name == property.name);
        match (existing, property.name) {
            (Some(own), "className") => {
                if let (PropertyValue::List(items), PropertyValue::List(old_items)) =
                    (&mut own.value, property.value)
                {
                    for item in old_items {
                        if !items.contains(&item) {
                            items.push(item);
                        }
                    }
                }
            }
            (Some(_), _) => {}
            (None, _) => new.properties.push(property),
        }
    }
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
