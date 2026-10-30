use textlane_core::Allocator;
use textlane_core::nlcst::Node;

use super::parse_prose;
use crate::models::options::{Language, Parse};

fn count_sentences(root: &Node<'_>) -> usize {
    let mut count = 0;
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if matches!(node, Node::Sentence(_)) {
            count += 1;
        }
        stack.extend(node.children().into_iter().flatten());
    }
    count
}

#[test]
fn keeps_abbreviations_in_their_sentence() {
    let alloc = Allocator::new();
    let input = "Ask Mr. Smith. He knows.";
    let latin = parse_prose(input, &Parse::default(), &alloc);
    let english = parse_prose(
        input,
        &Parse {
            language: Language::English,
        },
        &alloc,
    );
    assert_eq!(count_sentences(&latin), 3);
    assert_eq!(count_sentences(&english), 2);
}

#[test]
fn reads_astral_characters_as_one_symbol() {
    let alloc = Allocator::new();
    let root = parse_prose("a 😀😀", &Parse::default(), &alloc);
    let symbols: Vec<_> = {
        let mut found = Vec::new();
        let mut stack = vec![&root];
        while let Some(node) = stack.pop() {
            if let Node::Symbol(symbol) = node {
                found.push(symbol.value);
            }
            stack.extend(node.children().into_iter().flatten());
        }
        found
    };
    assert_eq!(symbols, ["😀😀"]);
}
