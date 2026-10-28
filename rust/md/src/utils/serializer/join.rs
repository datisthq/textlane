use textlane_core::mdast::Node;

use crate::utils::serializer::format::{
    format_code_as_indented, format_heading_as_setext, read_html_kind,
};
use crate::utils::serializer::settings::Settings;
use crate::utils::serializer::state::{Parent, Serializer};

/// What goes between two flow siblings, as mdast-util-to-markdown's `join` rules decide:
/// later rules first, `tightDefinitions` before the defaults.
pub(crate) fn join_siblings(
    state: &Serializer,
    left: &Node,
    right: &Node,
    parent: Parent,
) -> &'static str {
    let mut rules: Vec<Option<Join>> = vec![join_defaults(state, left, right, parent)];
    if state.settings.tight_definitions {
        rules.push(join_definition(left, right));
    }
    if state.settings.definition_list {
        rules.push(join_def_list(left, right, parent));
    }
    match rules.into_iter().rev().flatten().next() {
        Some(Join::Lines(0)) => "\n",
        Some(Join::Lines(2..)) => "\n\n\n",
        Some(Join::Comment) => "\n\n<!---->\n\n",
        Some(Join::Lines(1)) | None => "\n\n",
    }
}

// How many blank lines go between two siblings, or a comment when no number of them would
// keep the siblings apart
#[derive(Clone, Copy)]
enum Join {
    Lines(u8),
    Comment,
}

fn join_definition(left: &Node, right: &Node) -> Option<Join> {
    matches!((left, right), (Node::Definition(_), Node::Definition(_)))
        .then_some(Join::Lines(0))
}

// In a definition list, a blank line before a term after a description or before a
// spread description, none otherwise
fn join_def_list(left: &Node, right: &Node, parent: Parent) -> Option<Join> {
    if !matches!(parent.node, Some(Node::DefList(_))) {
        return None;
    }
    let blank = match right {
        Node::DefListTerm(_) => matches!(left, Node::DefListDescription(_)),
        Node::DefListDescription(description) => description.spread,
        _ => false,
    };
    Some(Join::Lines(u8::from(blank)))
}

// mdast-util-to-markdown's defaults inside any parent with a boolean `spread`
fn join_spread(
    left: &Node,
    right: &Node,
    spread: bool,
    settings: &Settings,
) -> Option<Join> {
    if matches!(left, Node::Paragraph(_))
        && match right {
            Node::Definition(_) | Node::Paragraph(_) => true,
            Node::Heading(heading) => format_heading_as_setext(heading, settings),
            _ => false,
        }
    {
        return None;
    }
    if let Node::Html(html) = left
        && matches!(read_html_kind(html.value), None | Some(6 | 7))
    {
        return Some(Join::Lines(1));
    }
    if matches!(left, Node::Paragraph(_))
        && let Node::Html(html) = right
        && matches!(read_html_kind(html.value), None | Some(7))
    {
        return Some(Join::Lines(1));
    }
    Some(Join::Lines(u8::from(spread)))
}

fn join_defaults(
    state: &Serializer,
    left: &Node,
    right: &Node,
    parent: Parent,
) -> Option<Join> {
    let settings = state.settings;
    let is_indented_code = |node: &Node| matches!(node, Node::Code(code) if format_code_as_indented(code, settings));
    if is_indented_code(right)
        && (matches!(left, Node::List(_)) || is_indented_code(left))
    {
        return Some(Join::Comment);
    }
    let spread = match parent.node {
        Some(Node::List(list)) => list.spread?,
        Some(Node::ListItem(item)) => item.spread?,
        Some(Node::DefListDescription(description)) => {
            return join_spread(left, right, description.spread, settings);
        }
        _ => return None,
    };
    if spread {
        return Some(Join::Lines(1));
    }
    if let Node::Html(html) = left
        && matches!(read_html_kind(html.value), None | Some(6 | 7))
    {
        return Some(Join::Lines(1));
    }
    if matches!((left, right), (Node::Blockquote(_), Node::Blockquote(_))) {
        return Some(Join::Lines(1));
    }
    if matches!(left, Node::Definition(_) | Node::Paragraph(_))
        && let Node::List(list) = right
        && ((list.ordered && list.start.is_some_and(|start| start != 1))
            || list.children.first().is_some_and(|item| {
                item.children().is_none_or(|children| children.is_empty())
            }))
    {
        return Some(Join::Lines(1));
    }
    if matches!((left, right), (Node::Paragraph(_), Node::ThematicBreak(_)))
        && settings.rule == '-'
        && !settings.rule_spaces
    {
        return Some(Join::Lines(1));
    }
    let mut tail = left;
    while matches!(
        tail,
        Node::Blockquote(_) | Node::List(_) | Node::ListItem(_)
    ) && let Some(last) = tail.children().and_then(|children| children.last())
    {
        tail = last;
    }
    if !matches!(tail, Node::Definition(_) | Node::Paragraph(_)) {
        return Some(Join::Lines(0));
    }
    if is_indented_code(right) {
        return Some(Join::Lines(1));
    }
    if let Node::Html(html) = right
        && matches!(read_html_kind(html.value), None | Some(7))
    {
        return Some(Join::Lines(1));
    }
    if (matches!(left, Node::Paragraph(_)) || !std::ptr::eq(left, tail))
        && match right {
            Node::Definition(_) | Node::Paragraph(_) => true,
            Node::Heading(heading) => format_heading_as_setext(heading, settings),
            _ => false,
        }
    {
        return Some(Join::Lines(1));
    }
    Some(Join::Lines(0))
}
