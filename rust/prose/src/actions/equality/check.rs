use std::collections::HashMap;

use textlane_core::Allocator;
use textlane_core::nlcst::Node;

use crate::actions::nlcst::search::search_nlcst;
use crate::actions::nlcst::stringify::stringify_nlcst;
use crate::actions::nlcst::visit::visit_nlcst;
use crate::helpers::equality_table::{EqualityPattern, PATTERNS};
use crate::helpers::normalize::normalize_word;
use crate::helpers::place::join_positions;
use crate::models::equality::EqualityRules;
use crate::models::report::Report;

const SOURCE: &str = "retext-equality";

struct Match<'t, 'a> {
    siblings: &'t [Node<'a>],
    parent: usize,
    start: usize,
    end: usize,
    kind: &'static str,
}

/// retext-equality's warnings about a tree: each paragraph searched for the patterns'
/// phrases, then reported pattern by pattern.
pub(crate) fn check_equality<'a>(
    tree: &Node<'a>,
    rules: &EqualityRules,
    alloc: &'a Allocator,
) -> Vec<Report<'a>> {
    let mut messages = Vec::new();
    let is_paragraph = |node: &Node<'_>| matches!(node, Node::Paragraph(_));
    visit_nlcst(tree, is_paragraph, true, |paragraph| {
        let mut slots = HashMap::new();
        let mut by_pattern: Vec<(usize, Vec<Match>)> = Vec::new();
        for (phrases, entries) in &rules.lists {
            search_nlcst(paragraph, phrases, |found| {
                let entry = entries[found.phrase];
                let nodes = &found.siblings[found.start..found.end];
                if entry.phrase != entry.phrase.to_lowercase()
                    && join_nodes(nodes) != entry.phrase
                {
                    return;
                }
                let slot = *slots.entry(entry.pattern).or_insert_with(|| {
                    by_pattern.push((entry.pattern, Vec::new()));
                    by_pattern.len() - 1
                });
                by_pattern[slot].1.push(Match {
                    siblings: found.siblings,
                    parent: found.parent,
                    start: found.start,
                    end: found.end - 1,
                    kind: entry.kind,
                });
            });
        }
        for (pattern, matches) in &by_pattern {
            let pattern = &PATTERNS[*pattern];
            if rules.binary && pattern.or {
                report_or(matches, pattern, alloc, &mut messages);
            } else {
                for found in matches {
                    messages.push(create_warning(found, pattern, alloc));
                }
            }
        }
    });
    messages
}

// A match right before one of the other category, with only white space, `and`, `or` or `/`
// between (`he or she`), is fine
fn report_or<'a>(
    matches: &[Match<'_, 'a>],
    pattern: &EqualityPattern,
    alloc: &'a Allocator,
    messages: &mut Vec<Report<'a>>,
) {
    let mut index = 0;
    while index < matches.len() {
        let found = &matches[index];
        if let Some(next) = matches.get(index + 1)
            && next.parent == found.parent
            && next.kind != found.kind
        {
            let mut start = found.end + 1;
            while start < next.start {
                let sibling = &found.siblings[start];
                let joining = match sibling {
                    Node::WhiteSpace(_) => true,
                    Node::Word(_) => {
                        let word = normalize_word(&stringify_nlcst(sibling), false);
                        word.contains("and") || word.contains("or")
                    }
                    Node::Punctuation(_) => {
                        normalize_word(&stringify_nlcst(sibling), false) == "/"
                    }
                    _ => false,
                };
                if !joining {
                    break;
                }
                start += 1;
            }
            if start == next.start {
                index += 2;
                continue;
            }
        }
        messages.push(create_warning(found, pattern, alloc));
        index += 1;
    }
}

fn create_warning<'a>(
    found: &Match<'_, 'a>,
    pattern: &EqualityPattern,
    alloc: &'a Allocator,
) -> Report<'a> {
    let nodes = &found.siblings[found.start..=found.end];
    let actual = join_nodes(nodes);
    let capitalize = actual
        .chars()
        .next()
        .is_some_and(|first| first.to_uppercase().eq(std::iter::once(first)));
    let expected: Vec<String> = pattern
        .considerate
        .iter()
        .map(|word| match word.chars().next() {
            Some(first) if capitalize => {
                first.to_uppercase().chain(word.chars().skip(1)).collect()
            }
            _ => (*word).to_string(),
        })
        .map(|word| format!("`{word}`"))
        .collect();
    let mut reason = format!("Unexpected potentially insensitive use of `{actual}`");
    if let Some(condition) = pattern.condition {
        reason.push_str(", ");
        reason.push_str(condition);
    }
    if expected.is_empty() {
        reason.push_str(", try not to use it");
    } else {
        reason.push_str(&format!(
            ", in somes cases {} may be better",
            expected.join(", ")
        ));
    }
    let position = match (nodes.first(), nodes.last()) {
        (Some(first), Some(last)) => join_positions(first, last, alloc),
        _ => None,
    };
    Report {
        reason,
        rule: pattern.id.to_string(),
        source: SOURCE,
        position,
    }
}

fn join_nodes(nodes: &[Node<'_>]) -> String {
    nodes.iter().map(stringify_nlcst).collect()
}
