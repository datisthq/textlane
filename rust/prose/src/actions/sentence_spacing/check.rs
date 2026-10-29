use textlane_core::Allocator;
use textlane_core::nlcst::Node;

use crate::actions::nlcst::stringify::stringify_nlcst;
use crate::actions::nlcst::visit::visit_nlcst;
use crate::helpers::place::join_positions;
use crate::models::report::Report;

const SOURCE: &str = "retext-sentence-spacing";

/// retext-sentence-spacing's warnings: spaces between two sentences of a paragraph that are
/// not the `expected` count (`0` asks for a line ending instead).
pub(crate) fn check_sentence_spacing<'a>(
    tree: &Node<'a>,
    expected: usize,
    alloc: &'a Allocator,
) -> Vec<Report<'a>> {
    let mut messages = Vec::new();
    let is_paragraph = |node: &Node<'_>| matches!(node, Node::Paragraph(_));
    visit_nlcst(tree, is_paragraph, false, |paragraph| {
        let Some(children) = paragraph.children() else {
            return;
        };
        for window in children.windows(3) {
            let [
                Node::Sentence(_),
                child @ Node::WhiteSpace(_),
                Node::Sentence(_),
            ] = window
            else {
                continue;
            };
            let value = stringify_nlcst(child);
            // Only spaces: a line ending is fine wherever a space is
            if value.is_empty() || value.bytes().any(|byte| byte != b' ') {
                continue;
            }
            let actual = value.len();
            if actual == expected {
                continue;
            }
            let plural = |count: usize| if count == 1 { "" } else { "s" };
            let reason = if expected == 0 {
                "Unexpected spaces between sentence, expected a line ending".to_string()
            } else {
                format!(
                    "Unexpected {actual} space{} between sentence, expected {expected} space{}",
                    plural(actual),
                    plural(expected)
                )
            };
            let rule = match expected {
                0 => "newline",
                1 => "space",
                _ => "double-space",
            };
            messages.push(Report {
                reason,
                rule: rule.to_string(),
                source: SOURCE,
                position: join_positions(child, child, alloc),
            });
        }
    });
    messages
}
