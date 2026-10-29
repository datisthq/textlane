use textlane_core::Allocator;
use textlane_core::nlcst::Node;

use crate::actions::nlcst::stringify::stringify_nlcst;
use crate::actions::nlcst::visit::visit_nlcst;
use crate::helpers::place::join_positions;
use crate::models::report::Report;

const SOURCE: &str = "retext-repeated-words";
// Words that can rightly occur twice in a row: `they had had`
const ALLOWED: [&str; 9] = [
    "had", "that", "can", "blah", "beep", "yadda", "sapiens", "tse", "mau",
];

/// retext-repeated-words' warnings: a word repeated with only white space between, in each
/// sentence.
pub(crate) fn check_repeated_words<'a>(
    tree: &Node<'a>,
    alloc: &'a Allocator,
) -> Vec<Report<'a>> {
    let mut messages = Vec::new();
    let is_sentence = |node: &Node<'_>| matches!(node, Node::Sentence(_));
    visit_nlcst(tree, is_sentence, true, |sentence| {
        let Some(children) = sentence.children() else {
            return;
        };
        let mut previous: Option<(usize, String)> = None;
        let mut current: Option<(usize, String)> = None;
        for (index, child) in children.iter().enumerate() {
            match child {
                Node::Word(_) => {
                    let value = stringify_nlcst(child);
                    let caseless = value.to_lowercase();
                    if previous.as_ref().is_some_and(|(_, word)| *word == caseless)
                        && !is_allowed(&value)
                        && let Some((start, _)) = previous
                    {
                        let position = join_positions(&children[start], child, alloc);
                        let reason = format!(
                            "Unexpected repeated `{value}`, remove one occurrence"
                        );
                        let rule = replace_non_word(&caseless);
                        messages.push(Report {
                            reason,
                            rule,
                            source: SOURCE,
                            position,
                        });
                    }
                    current = Some((index, caseless));
                }
                Node::WhiteSpace(_) => previous = current.take(),
                _ => {
                    previous = None;
                    current = None;
                }
            }
        }
    });
    messages
}

// Names, initials and the listed words: `Duran Duran`, `D. D.`
fn is_allowed(value: &str) -> bool {
    if ALLOWED.contains(&value.to_lowercase().as_str()) {
        return true;
    }
    let Some(head) = value.chars().next() else {
        return true;
    };
    // `charAt(0)` of an astral character is half of it, which no case changes
    let astral = head.len_utf16() == 2;
    if !astral && !head.to_uppercase().eq(std::iter::once(head)) {
        return false;
    }
    if value.encode_utf16().count() == 2 && value[head.len_utf8()..] == *"." {
        return true;
    }
    let tail = &value[head.len_utf8()..];
    tail == tail.to_lowercase()
}

// `/\W+/g` into `-`
fn replace_non_word(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut in_run = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            result.push(character);
            in_run = false;
        } else if !in_run {
            result.push('-');
            in_run = true;
        }
    }
    result
}
