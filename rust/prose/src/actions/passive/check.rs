use textlane_core::Allocator;
use textlane_core::nlcst::Node;

use crate::actions::nlcst::search::search_nlcst;
use crate::actions::nlcst::stringify::stringify_nlcst;
use crate::helpers::place::join_positions;
use crate::models::phrases::Phrases;
use crate::models::report::Report;

const SOURCE: &str = "retext-passive";
const VERBS: [&str; 8] = ["am", "are", "were", "being", "is", "been", "was", "be"];

/// retext-passive's warnings: each participle whose closest word before is a form of `be`.
pub(crate) fn check_passive<'a>(
    tree: &Node<'a>,
    participles: &[&str],
    phrases: &Phrases,
    alloc: &'a Allocator,
) -> Vec<Report<'a>> {
    let mut messages = Vec::new();
    search_nlcst(tree, phrases, |found| {
        let before = found.siblings[..found.start]
            .iter()
            .rev()
            .find(|sibling| matches!(sibling, Node::Word(_)));
        let Some(before) = before else {
            return;
        };
        if !VERBS.contains(&stringify_nlcst(before).to_lowercase().as_str()) {
            return;
        }
        let nodes = &found.siblings[found.start..found.end];
        let position = match (nodes.first(), nodes.last()) {
            (Some(first), Some(last)) => join_positions(first, last, alloc),
            _ => None,
        };
        let phrase = participles[found.phrase];
        let rule = phrase
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("-")
            .to_lowercase();
        let reason = "Unexpected use of the passive voice".to_string();
        messages.push(Report {
            reason,
            rule,
            source: SOURCE,
            position,
        });
    });
    messages
}
