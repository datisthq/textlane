use textlane_core::mdast::{MdxJsxAttribute, MdxJsxAttributeValue, MdxJsxElement, Node};
use textlane_core::{Allocator, ArenaVec, Position};

use crate::models::options::DirectiveRule;
use crate::plugins::component::actions::element::create::create_element;

/// Wrap each run of adjacent siblings one grouping rule converted, `rules[index]` per child,
/// in the rule's group component once the run is at least `min` long.
pub(crate) fn group_children<'a>(
    rules: &[DirectiveRule],
    children: &mut ArenaVec<'a, Node<'a>>,
    marks: &[Option<usize>],
    alloc: &'a Allocator,
) {
    let grouped =
        |mark: Option<usize>| mark.filter(|&index| rules[index].group.is_some());
    let nodes =
        std::mem::replace(children, ArenaVec::with_capacity_in(marks.len(), alloc));
    let mut run: Vec<Node<'a>> = Vec::new();
    let mut run_rule = None;
    for (node, &mark) in nodes.into_iter().zip(marks) {
        let mark = grouped(mark);
        if mark.is_none() || mark != run_rule {
            flush_run(rules, run_rule, &mut run, children, alloc);
        }
        run_rule = mark;
        match mark {
            Some(_) => run.push(node),
            None => children.push(node),
        }
    }
    flush_run(rules, run_rule, &mut run, children, alloc);
}

fn flush_run<'a>(
    rules: &[DirectiveRule],
    rule: Option<usize>,
    run: &mut Vec<Node<'a>>,
    children: &mut ArenaVec<'a, Node<'a>>,
    alloc: &'a Allocator,
) {
    let group = rule.and_then(|index| rules[index].group.as_ref());
    let Some(group) = group.filter(|group| run.len() >= group.min.unwrap_or(2)) else {
        children.extend(run.drain(..));
        return;
    };
    let first = run.first();
    let attributes = group
        .inherit
        .iter()
        .filter_map(|name| {
            let value = first.and_then(|node| read_attribute(node, name))?;
            Some((name.clone(), value.to_string()))
        })
        .collect();
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
    let name = alloc.alloc_str(&group.component);
    children.push(create_element(
        name, attributes, items, position, text, alloc,
    ));
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
