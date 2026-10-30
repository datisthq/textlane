use textlane_core::Error;
use textlane_core::mdast::Node;

use crate::helpers::character::CharacterKind;
use crate::models::options::Directive;
use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::state::{Info, Parent, Serializer, classify_unit};

const SEQUENCE_SIZE_MIN: usize = 3;

/// Write a directive: its fence, name, label and attributes, then a container's content
/// and closing fence.
pub(crate) fn stringify_directive(
    state: &mut Serializer,
    node: &Node,
    info: Info,
) -> Result<(), Error> {
    let (name, attributes, construct, label_construct) = match node {
        Node::ContainerDirective(directive) => (
            directive.name,
            &directive.attributes,
            Construct::ContainerDirective,
            Construct::ContainerDirectiveLabel,
        ),
        Node::LeafDirective(directive) => (
            directive.name,
            &directive.attributes,
            Construct::LeafDirective,
            Construct::LeafDirectiveLabel,
        ),
        Node::TextDirective(directive) => (
            directive.name,
            &directive.attributes,
            Construct::TextDirective,
            Construct::TextDirectiveLabel,
        ),
        _ => return Ok(()),
    };
    let children = node.children().map_or(&[][..], |children| children);
    let container = matches!(node, Node::ContainerDirective(_));
    let head_label = match node {
        Node::ContainerDirective(directive) if directive.label => children
            .first()
            .filter(|head| matches!(head, Node::Paragraph(_))),
        _ => None,
    };
    let label = if container {
        head_label.map(Parent::of)
    } else {
        Some(Parent::of(node))
    };
    let sequence = ":".repeat(measure_fence(node));
    state.enter(construct);
    let mut value = format!("{sequence}{name}");
    let attributes_value = serialize_attributes(state, node, attributes);
    let after = info.after.chars().next();
    if let Some(label) = label.filter(|label| !label.children.is_empty()) {
        state.enter(Construct::Label);
        state.enter(label_construct);
        value.push('[');
        let text = state.container_phrasing(
            label,
            Info {
                before: &value,
                after: "]",
            },
        );
        state.exit();
        state.exit();
        match text {
            Ok(text) => value.push_str(&text),
            Err(error) => {
                state.exit();
                return Err(error);
            }
        }
        value.push(']');
    } else if attributes_value.is_empty()
        && (matches!(after, Some('-' | '_'))
            || classify_unit(after) == CharacterKind::Other)
    {
        value.push_str("[]");
    }
    if !attributes_value.is_empty() {
        value.push_str(&attributes_value);
    } else if after == Some('{') {
        value.push_str("{}");
    }
    if container {
        let content = &children[usize::from(head_label.is_some())..];
        if !content.is_empty() {
            value.push('\n');
            state.write(&value);
            value.clear();
            let parent = Parent {
                node: Some(node),
                children: content,
            };
            if let Err(error) = state.container_flow(parent) {
                state.exit();
                return Err(error);
            }
        }
        value.push('\n');
        value.push_str(&sequence);
    }
    state.exit();
    state.write(&value);
    Ok(())
}

// A container's fence is longer than any container inside it, by how deeply they nest
fn measure_fence(node: &Node) -> usize {
    match node {
        Node::ContainerDirective(_) => {}
        Node::LeafDirective(_) => return 2,
        _ => return 1,
    }
    let mut size = 0;
    let mut stack = vec![(node, 0)];
    while let Some((node, depth)) = stack.pop() {
        let nested = matches!(node, Node::ContainerDirective(_));
        if nested {
            size = size.max(depth);
        }
        for child in node.children().into_iter().flatten() {
            stack.push((child, depth + usize::from(nested)));
        }
    }
    size + SEQUENCE_SIZE_MIN
}

fn serialize_attributes(
    state: &Serializer,
    node: &Node,
    attributes: &[(&str, &str)],
) -> String {
    let defaults = Directive::default();
    let options = state.settings.directive.as_ref().unwrap_or(&defaults);
    let mut values = Vec::new();
    let mut id = None;
    let mut classes = String::new();
    let mut classes_full = String::new();
    for (key, value) in order_attributes(attributes) {
        if key == "id" {
            id = Some(if options.prefer_shortcut && is_shortcut(value) {
                format!("#{value}")
            } else {
                quote_attribute(state, options, node, key, value)
            });
        } else if key == "class" {
            let mut shortcuts = Vec::new();
            let mut full = Vec::new();
            for class in split_classes(value) {
                if options.prefer_shortcut && is_shortcut(class) {
                    shortcuts.push(class);
                } else {
                    full.push(class);
                }
            }
            classes_full = if full.is_empty() {
                String::new()
            } else {
                quote_attribute(state, options, node, "class", &full.join(" "))
            };
            classes = if shortcuts.is_empty() {
                String::new()
            } else {
                format!(".{}", shortcuts.join("."))
            };
        } else {
            values.push(quote_attribute(state, options, node, key, value));
        }
    }
    if !classes_full.is_empty() {
        values.insert(0, classes_full);
    }
    if !classes.is_empty() {
        values.insert(0, classes);
    }
    if let Some(id) = id {
        values.insert(0, id);
    }
    if values.is_empty() {
        String::new()
    } else {
        format!("{{{}}}", values.join(" "))
    }
}

// JavaScript walks an object's keys that are array indices first, in numeric order
fn order_attributes<'v>(attributes: &'v [(&'v str, &'v str)]) -> Vec<(&'v str, &'v str)> {
    let index = |key: &str| {
        let canonical = key == "0" || (!key.starts_with('0') && !key.is_empty());
        key.parse::<u32>().ok().filter(|index| {
            canonical
                && *index < u32::MAX
                && key.bytes().all(|byte| byte.is_ascii_digit())
        })
    };
    let mut ordered: Vec<(&str, &str)> = attributes.to_vec();
    ordered.sort_by_key(|(key, _)| match index(key) {
        Some(index) => (0, index),
        None => (1, 0),
    });
    ordered
}

// JavaScript's `split(/[\t\n\r ]+/)`: a run of whitespace separates, and one at either edge
// leaves an empty class there
fn split_classes(value: &str) -> Vec<&str> {
    let is_space = |character: char| matches!(character, '\t' | '\n' | '\r' | ' ');
    let mut classes = Vec::new();
    let mut start = 0;
    let mut chars = value.char_indices().peekable();
    while let Some((index, character)) = chars.next() {
        if is_space(character) {
            classes.push(&value[start..index]);
            let mut end = index + character.len_utf8();
            while let Some(&(next, character)) = chars.peek() {
                if !is_space(character) {
                    break;
                }
                end = next + character.len_utf8();
                chars.next();
            }
            start = end;
        }
    }
    classes.push(&value[start..]);
    classes
}

fn is_shortcut(value: &str) -> bool {
    !value.is_empty()
        && !value.contains([
            '\t', '\n', '\r', ' ', '"', '#', '\'', '.', '<', '=', '>', '`', '}',
        ])
}

fn is_unquoted(value: &str) -> bool {
    !value.is_empty()
        && !value.contains(['\t', '\n', '\r', ' ', '"', '\'', '<', '=', '>', '`', '}'])
}

fn quote_attribute(
    state: &Serializer,
    options: &Directive,
    node: &Node,
    key: &str,
    value: &str,
) -> String {
    if options.collapse_empty_attributes && value.is_empty() {
        return key.to_string();
    }
    if options.prefer_unquoted && is_unquoted(value) {
        return format!("{key}={value}");
    }
    let preferred = options
        .quote
        .as_deref()
        .and_then(|quote| quote.chars().next())
        .unwrap_or(state.settings.quote);
    let alternative = if preferred == '"' { '\'' } else { '"' };
    let count = |quote: char| {
        value
            .chars()
            .filter(|character| *character == quote)
            .count()
    };
    let quote = if options.quote_smart && count(preferred) > count(alternative) {
        alternative
    } else {
        preferred
    };
    let text = matches!(node, Node::TextDirective(_));
    let mut result = format!("{key}={quote}");
    for character in value.chars() {
        if character == quote || (!text && matches!(character, '\n' | '\r')) {
            result.push_str(&format!("&#x{:X};", u32::from(character)));
        } else {
            result.push(character);
        }
    }
    result.push(quote);
    result
}
