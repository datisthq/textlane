use textlane_core::mdast::{MdxJsxAttribute, MdxJsxAttributeValue, MdxJsxElement, Node};
use textlane_core::{Allocator, ArenaVec, Position};

use crate::models::options::Component;
use crate::plugins::component::actions::element::create::create_element;
use crate::plugins::component::actions::meta::read::{read_meta, strip_meta};

/// The rule that converted or matched a sibling, which grouping reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mark {
    Directive(usize),
    Code(usize),
}

/// Wrap each run of adjacent siblings one grouping rule converted or matched, `marks` per
/// child, in the rule's group component once the run is at least `min` long.
pub(crate) fn group_children<'a>(
    options: &Component,
    children: &mut ArenaVec<'a, Node<'a>>,
    marks: &[Option<Mark>],
    alloc: &'a Allocator,
) {
    let grouped =
        |mark: Option<Mark>| mark.filter(|&mark| read_group(options, mark).is_some());
    let nodes =
        std::mem::replace(children, ArenaVec::with_capacity_in(marks.len(), alloc));
    let mut run: Vec<Node<'a>> = Vec::new();
    let mut run_mark = None;
    for (node, &mark) in nodes.into_iter().zip(marks) {
        let mark = grouped(mark);
        if mark.is_none() || mark != run_mark {
            flush_run(options, run_mark, &mut run, children, alloc);
        }
        run_mark = mark;
        match mark {
            Some(_) => run.push(node),
            None => children.push(node),
        }
    }
    flush_run(options, run_mark, &mut run, children, alloc);
}

/// Whether a mark's rule groups: its wrapper and shortest run.
pub(crate) fn read_group(options: &Component, mark: Mark) -> Option<(&str, usize)> {
    match mark {
        Mark::Directive(index) => options.directives[index]
            .group
            .as_ref()
            .map(|group| (group.component.as_str(), group.min.unwrap_or(2))),
        Mark::Code(index) => options.codes[index]
            .group
            .as_ref()
            .map(|group| (group.component.as_str(), group.min.unwrap_or(2))),
    }
}

fn flush_run<'a>(
    options: &Component,
    mark: Option<Mark>,
    run: &mut Vec<Node<'a>>,
    children: &mut ArenaVec<'a, Node<'a>>,
    alloc: &'a Allocator,
) {
    let group =
        mark.and_then(|mark| read_group(options, mark).map(|group| (mark, group)));
    let Some((mark, (component, _))) = group.filter(|(_, (_, min))| run.len() >= *min)
    else {
        children.extend(run.drain(..));
        return;
    };
    let attributes = match mark {
        Mark::Directive(index) => inherit_attributes(options, index, run),
        Mark::Code(index) => take_meta_attributes(options, index, run, alloc),
    };
    let first = run.first();
    let text = matches!(first, Some(Node::MdxJsxTextElement(_)));
    let position = match (
        first.and_then(Node::position),
        run.last().and_then(Node::position),
    ) {
        (Some(Position::Span { start, .. }), Some(Position::Span { end, .. })) => {
            Some(Position::span(*start, *end))
        }
        _ => None,
    };
    let mut items = ArenaVec::with_capacity_in(run.len(), alloc);
    items.extend(run.drain(..));
    let name = alloc.alloc_str(component);
    children.push(create_element(
        name, attributes, items, position, text, alloc,
    ));
}

fn inherit_attributes(
    options: &Component,
    index: usize,
    run: &[Node<'_>],
) -> Vec<(String, String)> {
    let Some(group) = &options.directives[index].group else {
        return Vec::new();
    };
    group
        .inherit
        .iter()
        .filter_map(|name| {
            let value = run.first().and_then(|node| read_attribute(node, name))?;
            Some((name.clone(), value.to_string()))
        })
        .collect()
}

// A code group's wrapper takes the run's values of the rule's key and the first block's
// inherited keys, which leave every block's meta
fn take_meta_attributes<'a>(
    options: &Component,
    index: usize,
    run: &mut [Node<'a>],
    alloc: &'a Allocator,
) -> Vec<(String, String)> {
    let rule = &options.codes[index];
    let Some(group) = &rule.group else {
        return Vec::new();
    };
    let metas: Vec<&str> = run
        .iter()
        .map(|node| match node {
            Node::Code(code) => code.meta.unwrap_or_default(),
            _ => "",
        })
        .collect();
    let mut attributes = Vec::new();
    if let (Some(name), Some(key)) = (&group.collect, &rule.meta) {
        let values: Vec<String> = metas
            .iter()
            .filter_map(|meta| read_meta(meta, key))
            .collect();
        let list = serde_json::to_string(&values).unwrap_or_default();
        attributes.push((name.clone(), list));
    }
    for key in &group.inherit {
        if let Some(value) = metas.first().and_then(|meta| read_meta(meta, key)) {
            attributes.push((key.clone(), value));
        }
    }
    let mut keys: Vec<&str> = rule.meta.iter().map(String::as_str).collect();
    keys.extend(group.inherit.iter().map(String::as_str));
    for node in run.iter_mut() {
        if let Node::Code(code) = node
            && let Some(meta) = code.meta
        {
            code.meta = strip_meta(meta, &keys).map(|meta| &*alloc.alloc_str(&meta));
        }
    }
    attributes
}

fn read_attribute<'a>(node: &Node<'a>, name: &str) -> Option<&'a str> {
    let (Node::MdxJsxFlowElement(MdxJsxElement { attributes, .. })
    | Node::MdxJsxTextElement(MdxJsxElement { attributes, .. })) = node
    else {
        return None;
    };
    attributes.iter().find_map(|attribute| match attribute {
        MdxJsxAttribute::Attribute {
            name: key,
            value: Some(MdxJsxAttributeValue::Literal(value)),
            ..
        } if *key == name => Some(*value),
        _ => None,
    })
}
