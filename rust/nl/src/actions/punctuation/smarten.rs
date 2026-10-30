use textlane_core::nlcst::Node;
use textlane_core::{Allocator, ArenaVec};

use crate::actions::nlcst::stringify::stringify_nlcst;
use crate::helpers::node::{children_mut, is_decade, is_mark, mark_value, set_value};
use crate::models::smartypants::{Backticks, Dashes, Ellipses};

/// What `nl.smartypants` rewrites, its quotes allocated in the document's arena.
pub(crate) struct Smarten<'a> {
    pub quotes: bool,
    pub ellipses: Ellipses,
    pub backticks: Backticks,
    pub dashes: Dashes,
    /// The double and the single opening quote.
    pub open: [&'a str; 2],
    pub close: [&'a str; 2],
}

/// Replace straight punctuation with typographic punctuation, as retext-smartypants does.
pub(crate) fn smarten_punctuation<'a>(
    tree: &mut Node<'a>,
    settings: &Smarten<'a>,
    alloc: &'a Allocator,
) {
    let mut stack = vec![tree];
    while let Some(node) = stack.pop() {
        let Some(children) = children_mut(node) else {
            continue;
        };
        if !children.iter().any(is_mark) {
            stack.extend(children.iter_mut());
            continue;
        }
        let visited = smarten_children(children, settings, alloc);
        stack.extend(
            children
                .iter_mut()
                .zip(visited)
                .filter_map(|(child, visited)| visited.then_some(child)),
        );
    }
}

// Upstream splices siblings out while unist-util-visit walks them by index; rebuilding the
// list keeps that linear. Its quirks are kept: after an ellipsis drops the dots and spaces
// before it, the walk resumes as many siblings later, never visiting those
fn smarten_children<'a>(
    children: &mut ArenaVec<'a, Node<'a>>,
    settings: &Smarten<'a>,
    alloc: &'a Allocator,
) -> Vec<bool> {
    let mut rest: Vec<Node<'a>> = std::mem::replace(children, ArenaVec::new_in(alloc))
        .into_iter()
        .collect();
    rest.reverse();
    let done = children;
    done.reserve(rest.len());
    let mut visited = Vec::with_capacity(rest.len());
    let mut skip = 0;
    while let Some(mut node) = rest.pop() {
        let walked = skip == 0;
        if !walked {
            skip -= 1;
        } else if is_mark(&node) {
            let mut siblings = Siblings {
                done: &mut *done,
                visited: &mut visited,
                rest: &mut rest,
            };
            match smarten_mark(&mut node, &mut siblings, settings) {
                Outcome::Keep => {}
                Outcome::Skip(count) => skip = count,
                Outcome::Remove => continue,
            }
        }
        visited.push(walked);
        done.push(node);
    }
    visited
}

// The siblings around the mark being rewritten: those before it, whether the walk reached
// each, and those after it, nearest last
struct Siblings<'s, 'a> {
    done: &'s mut ArenaVec<'a, Node<'a>>,
    visited: &'s mut Vec<bool>,
    rest: &'s mut Vec<Node<'a>>,
}

impl<'a> Siblings<'_, 'a> {
    fn next(&self, offset: usize) -> Option<&Node<'a>> {
        self.rest
            .len()
            .checked_sub(offset + 1)
            .and_then(|index| self.rest.get(index))
    }
}

enum Outcome {
    Keep,
    /// The walk skips this many siblings after the mark.
    Skip(usize),
    Remove,
}

fn smarten_mark<'a>(
    node: &mut Node<'a>,
    siblings: &mut Siblings<'_, 'a>,
    settings: &Smarten<'a>,
) -> Outcome {
    let mut outcome = Outcome::Keep;
    if settings.quotes {
        smarten_quote(node, siblings, settings);
    }
    match settings.ellipses {
        Ellipses::Off => {}
        Ellipses::On => {
            outcome = smarten_spaced_ellipsis(node, siblings);
            smarten_unspaced_ellipsis(node);
        }
        Ellipses::Spaced => outcome = smarten_spaced_ellipsis(node, siblings),
        Ellipses::Unspaced => smarten_unspaced_ellipsis(node),
    }
    match (settings.backticks, mark_value(node)) {
        (Backticks::Off, _) => {}
        (_, Some("``")) => set_value(node, "“"),
        (_, Some("''")) => set_value(node, "”"),
        (Backticks::All, Some("`")) => set_value(node, "‘"),
        (Backticks::All, Some("'")) => set_value(node, "’"),
        _ => {}
    }
    let (merged, long, short) = match settings.dashes {
        Dashes::Off => return outcome,
        Dashes::On => return smarten_dash(node, None, "—", outcome),
        Dashes::Oldschool => ("–", "—", "–"),
        Dashes::Inverted => ("—", "–", "—"),
    };
    // A dash already turned into `merged` followed by a hyphen becomes the hyphen, rewritten
    if mark_value(node) == Some(merged)
        && let Some(next @ Node::Punctuation(_)) = siblings.rest.last_mut()
        && mark_value(next) == Some("-")
    {
        set_value(next, long);
        return Outcome::Remove;
    }
    smarten_dash(node, Some(long), short, outcome)
}

fn smarten_dash<'a>(
    node: &mut Node<'a>,
    three: Option<&'a str>,
    two: &'a str,
    outcome: Outcome,
) -> Outcome {
    match (mark_value(node), three) {
        (Some("---"), Some(three)) => set_value(node, three),
        (Some("--"), _) => set_value(node, two),
        _ => {}
    }
    outcome
}

fn smarten_quote<'a>(
    node: &mut Node<'a>,
    siblings: &mut Siblings<'_, 'a>,
    settings: &Smarten<'a>,
) {
    let value = mark_value(node);
    let quote = match value {
        Some("\"") => 0,
        Some("'") => 1,
        _ => return,
    };
    let previous = siblings.done.last();
    let next = siblings.next(0);
    let next_next = siblings.next(1);
    let next_value = next.map(stringify_nlcst).unwrap_or_default();
    let next_is_mark = next.is_some_and(is_mark);
    let next_next_is_word = matches!(next_next, Some(Node::Word(_)));
    let replacement = if next_is_mark && !next_next_is_word {
        // A quote before punctuation not starting a word closes
        settings.close[quote]
    } else if next_is_mark && (next_value == "\"" || next_value == "'") {
        // Two quotes before a word open twice: `"'Quoted' words"`
        let inner = usize::from(next_value == "'");
        if let Some(next) = siblings.rest.last_mut() {
            set_value(next, settings.open[inner]);
        }
        settings.open[quote]
    } else if next.is_some() && is_decade(&next_value) {
        settings.close[quote]
    } else if (matches!(previous, Some(Node::WhiteSpace(_)))
        || previous.is_some_and(is_mark))
        && matches!(next, Some(Node::Word(_)))
    {
        settings.open[quote]
    } else if previous.is_some_and(|previous| {
        !matches!(previous, Node::WhiteSpace(_)) && !is_mark(previous)
    }) || next.is_none()
        || matches!(next, Some(Node::WhiteSpace(_)))
        || (quote == 1 && next_value == "s")
    {
        settings.close[quote]
    } else {
        settings.open[quote]
    };
    set_value(node, replacement);
}

fn smarten_spaced_ellipsis(
    node: &mut Node<'_>,
    siblings: &mut Siblings<'_, '_>,
) -> Outcome {
    if !mark_value(node).is_some_and(is_dots) {
        return Outcome::Keep;
    }
    // Pairs of white space and dots before the node, as upstream's `while (--position > 0)`
    let done = &siblings.done;
    let mut position = done.len();
    let mut count = 1;
    let mut removed = 0;
    while position > 1 {
        position -= 1;
        if !matches!(done.get(position), Some(Node::WhiteSpace(_))) {
            break;
        }
        position -= 1;
        match done.get(position) {
            Some(sibling) if mark_value(sibling).is_some_and(is_dots) => {
                removed += 2;
                count += 1;
            }
            _ => break,
        }
    }
    if count < 3 {
        return Outcome::Keep;
    }
    let keep = siblings.done.len() - removed;
    siblings.done.truncate(keep);
    siblings.visited.truncate(keep);
    set_value(node, "…");
    Outcome::Skip(removed)
}

fn smarten_unspaced_ellipsis(node: &mut Node<'_>) {
    if mark_value(node).is_some_and(|value| value.len() >= 3 && is_dots(value)) {
        set_value(node, "…");
    }
}

fn is_dots(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte == b'.')
}
