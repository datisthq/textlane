use textlane_core::mdast::{
    ContainerDirective, DirectiveAttributes, LeafDirective, Node, TextDirective,
};
use textlane_core::{Allocator, ArenaVec, Data, Position};

use crate::models::options::{AttributeRule, DirectiveKind, DirectiveRule};
use crate::plugins::component::actions::element::create::create_element;
use crate::plugins::component::actions::text::read::read_text;
use crate::plugins::component::actions::tree::read::read_tree;

/// A directive of any kind, taken apart so a rule can rebuild it as an element.
pub(crate) struct DirectiveParts<'a> {
    pub(crate) kind: DirectiveKind,
    pub(crate) name: &'a str,
    pub(crate) attributes: DirectiveAttributes<'a>,
    pub(crate) label: bool,
    pub(crate) children: ArenaVec<'a, Node<'a>>,
    pub(crate) position: Option<Position<'a>>,
    pub(crate) data: Option<&'a Data<'a>>,
}

/// The element `rule` makes of a directive, or the directive back when the rule does not
/// match it: another name or kind, a `where` value or a `required` attribute missing.
pub(crate) fn convert_directive<'a>(
    rule: &DirectiveRule,
    mut parts: DirectiveParts<'a>,
    alloc: &'a Allocator,
) -> Result<Node<'a>, DirectiveParts<'a>> {
    if !matches_rule(rule, &parts) {
        return Err(parts);
    }
    let label = read_label(rule, &parts);
    let mut attributes = write_attributes(rule, &parts, label.as_deref());
    if let Some(name) = &rule.tree_as
        && parts.kind == DirectiveKind::Container
    {
        attributes.push((name.clone(), read_tree(&parts.children).to_string()));
    }
    let missing = |name: &String| {
        !attributes
            .iter()
            .any(|(key, value)| key == name && !value.is_empty())
    };
    if rule.required.iter().any(missing) {
        return Err(parts);
    }
    if rule.label_as.is_some() {
        drop_label(&mut parts);
    }
    if rule.tree_as.is_some() && parts.kind == DirectiveKind::Container {
        parts
            .children
            .retain(|child| !matches!(child, Node::List(_)));
    }
    let text = parts.kind == DirectiveKind::Text;
    Ok(create_element(
        alloc.alloc_str(&rule.component),
        attributes,
        parts.children,
        parts.position,
        text,
        alloc,
    ))
}

impl<'a> DirectiveParts<'a> {
    /// The parts of a directive node, or the node back when it is no directive.
    pub(crate) fn take(node: Node<'a>) -> Result<Self, Node<'a>> {
        Ok(match node {
            Node::ContainerDirective(node) => DirectiveParts {
                kind: DirectiveKind::Container,
                name: node.name,
                attributes: node.attributes,
                label: node.label,
                children: node.children,
                position: node.position,
                data: node.data,
            },
            Node::LeafDirective(node) => DirectiveParts {
                kind: DirectiveKind::Leaf,
                name: node.name,
                attributes: node.attributes,
                label: false,
                children: node.children,
                position: node.position,
                data: node.data,
            },
            Node::TextDirective(node) => DirectiveParts {
                kind: DirectiveKind::Text,
                name: node.name,
                attributes: node.attributes,
                label: false,
                children: node.children,
                position: node.position,
                data: node.data,
            },
            node => return Err(node),
        })
    }

    /// The directive node again.
    pub(crate) fn into_node(self) -> Node<'a> {
        match self.kind {
            DirectiveKind::Container => Node::ContainerDirective(ContainerDirective {
                name: self.name,
                attributes: self.attributes,
                label: self.label,
                children: self.children,
                position: self.position,
                data: self.data,
            }),
            DirectiveKind::Leaf => Node::LeafDirective(LeafDirective {
                name: self.name,
                attributes: self.attributes,
                children: self.children,
                position: self.position,
                data: self.data,
            }),
            DirectiveKind::Text => Node::TextDirective(TextDirective {
                name: self.name,
                attributes: self.attributes,
                children: self.children,
                position: self.position,
                data: self.data,
            }),
        }
    }

    /// The last value written for an attribute, as mdast-util-directive keeps it.
    pub(crate) fn attribute(&self, name: &str) -> Option<&'a str> {
        let found = self.attributes.iter().rev().find(|(key, _)| *key == name);
        found.map(|(_, value)| *value)
    }
}

fn matches_rule(rule: &DirectiveRule, parts: &DirectiveParts<'_>) -> bool {
    rule.name.as_slice().iter().any(|name| name == parts.name)
        && rule.kind.is_none_or(|kind| kind == parts.kind)
        && rule
            .condition
            .iter()
            .all(|(name, value)| parts.attribute(name) == Some(value.as_str()))
}

// A container's label is its label paragraph; a text or leaf directive's is its content,
// read only when the rule takes it
fn read_label(rule: &DirectiveRule, parts: &DirectiveParts<'_>) -> Option<String> {
    let text = match parts.kind {
        DirectiveKind::Container if parts.label => parts.children.first().map(read_text),
        DirectiveKind::Container => None,
        _ if rule.label_as.is_some() => {
            Some(parts.children.iter().map(read_text).collect::<String>())
        }
        _ => None,
    };
    text.filter(|text| !text.is_empty())
        .or_else(|| rule.label_default.clone())
}

fn drop_label(parts: &mut DirectiveParts<'_>) {
    match parts.kind {
        DirectiveKind::Container if parts.label && !parts.children.is_empty() => {
            parts.children.remove(0);
        }
        DirectiveKind::Container => {}
        _ => parts.children.clear(),
    }
}

// The name first, then the label unless a listed attribute places it, then the attributes:
// the order livemark's components were written against
fn write_attributes(
    rule: &DirectiveRule,
    parts: &DirectiveParts<'_>,
    label: Option<&str>,
) -> Vec<(String, String)> {
    let mut attributes = Vec::new();
    if let Some(name) = &rule.name_as {
        attributes.push((name.clone(), parts.name.to_string()));
    }
    let listed = |name: &str| match &rule.attributes {
        Some(rules) => rules.iter().any(|rule| read_output(rule) == name),
        None => parts.attributes.iter().any(|(key, _)| *key == name),
    };
    if let (Some(name), Some(label)) = (&rule.label_as, label)
        && !listed(name)
    {
        attributes.push((name.clone(), label.to_string()));
    }
    match &rule.attributes {
        Some(rules) => {
            for attribute in rules {
                let (name, default) = match attribute {
                    AttributeRule::Name(name) => (name, None),
                    AttributeRule::Rule { name, default, .. } => {
                        (name, default.as_deref())
                    }
                };
                let output = read_output(attribute);
                let from_label = (rule.label_as.as_deref() == Some(output))
                    .then_some(label)
                    .flatten();
                let value = from_label.or(parts.attribute(name)).or(default);
                if let Some(value) = value {
                    attributes.push((output.to_string(), value.to_string()));
                }
            }
        }
        None => {
            for (name, value) in parts.attributes.iter() {
                let value = match rule.label_as.as_deref() == Some(*name) {
                    true => label.unwrap_or(value),
                    false => value,
                };
                attributes.retain(|(key, _)| key != name);
                attributes.push((name.to_string(), value.to_string()));
            }
        }
    }
    attributes
}

fn read_output(rule: &AttributeRule) -> &str {
    match rule {
        AttributeRule::Name(name) => name,
        AttributeRule::Rule { name, rename, .. } => rename.as_deref().unwrap_or(name),
    }
}
