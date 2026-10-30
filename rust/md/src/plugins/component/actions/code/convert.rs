use textlane_core::mdast::{Code, Node, Text};
use textlane_core::{Allocator, ArenaVec};

use crate::models::options::{AttributeRule, CodeRule};
use crate::plugins::component::actions::element::create::create_element;
use crate::plugins::component::actions::meta::read::read_meta;

/// Whether a rule takes a code block: its language and its required meta key.
pub(crate) fn matches_code(rule: &CodeRule, code: &Code<'_>) -> bool {
    let lang = match &rule.lang {
        Some(names) => code
            .lang
            .is_some_and(|lang| names.as_slice().iter().any(|name| name == lang)),
        None => true,
    };
    let meta = match &rule.meta {
        Some(key) => code.meta.is_some_and(|meta| read_meta(meta, key).is_some()),
        None => true,
    };
    lang && meta
}

/// The component a rule with one makes of a code block: the code as its text, or as the
/// `valueAs` attribute, after the meta pairs the rule keeps.
pub(crate) fn convert_code<'a>(
    rule: &CodeRule,
    component: &str,
    code: Code<'a>,
    alloc: &'a Allocator,
) -> Node<'a> {
    let meta = code.meta.unwrap_or_default();
    let mut attributes = Vec::new();
    if let Some(name) = &rule.value_as {
        attributes.push((name.clone(), code.value.to_owned()));
    }
    for attribute in &rule.attributes {
        let (name, rename, default) = match attribute {
            AttributeRule::Name(name) => (name, None, None),
            AttributeRule::Rule {
                name,
                rename,
                default,
            } => (name, rename.as_ref(), default.as_ref()),
        };
        if let Some(value) = read_meta(meta, name).or_else(|| default.cloned()) {
            attributes.push((rename.unwrap_or(name).clone(), value));
        }
    }
    let mut children = ArenaVec::new_in(alloc);
    if rule.value_as.is_none() {
        children.push(Node::Text(Text {
            value: code.value,
            position: None,
            data: None,
        }));
    }
    create_element(
        alloc.alloc_str(component),
        attributes,
        children,
        code.position,
        false,
        alloc,
    )
}
