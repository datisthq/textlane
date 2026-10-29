use textlane_core::nlcst::Node;

use crate::actions::nlcst::stringify::stringify_nlcst;

const SINGLE: [&str; 5] = ["-", "–", "—", ":", ";"];
const PAIRS: [(&str, &[&str]); 21] = [
    (",", &[","]),
    ("-", &["-"]),
    ("–", &["–"]),
    ("—", &["—"]),
    ("\"", &["\""]),
    ("'", &["'"]),
    ("‘", &["’"]),
    ("‚", &["’"]),
    ("’", &["’", "‚"]),
    ("“", &["”"]),
    ("”", &["”"]),
    ("„", &["”", "“"]),
    ("«", &["»"]),
    ("»", &["«"]),
    ("‹", &["›"]),
    ("›", &["‹"]),
    ("(", &[")"]),
    ("[", &["]"]),
    ("{", &["}"]),
    ("⟨", &["⟩"]),
    ("「", &["」"]),
];

/// Whether the word at `index` is used as a literal, as nlcst-is-literal decides: set off by a
/// dash, colon or semicolon at a sentence edge, or wrapped in a pair of delimiters.
/// `words` is the first and last index of a word among `siblings`, so no check rescans them.
pub(crate) fn is_literal(
    siblings: &[Node<'_>],
    index: usize,
    words: Option<(usize, usize)>,
) -> bool {
    let word_before = words.is_some_and(|(first, _)| first < index);
    let word_after = words.is_some_and(|(_, last)| last > index);
    (!word_before && find_delimiter(siblings, index, true, &SINGLE).is_some())
        || (!word_after && find_delimiter(siblings, index, false, &SINGLE).is_some())
        || is_wrapped(siblings, index)
}

fn is_wrapped(siblings: &[Node<'_>], index: usize) -> bool {
    let opens = PAIRS.map(|(open, _)| open);
    let Some(open) = find_delimiter(siblings, index, false, &opens) else {
        return false;
    };
    PAIRS
        .iter()
        .find(|(candidate, _)| *candidate == open)
        .is_some_and(|(_, closes)| {
            find_delimiter(siblings, index, true, closes).is_some()
        })
}

// The first sibling past white space, when it is one of `delimiters`
fn find_delimiter(
    siblings: &[Node<'_>],
    index: usize,
    forward: bool,
    delimiters: &[&'static str],
) -> Option<&'static str> {
    let mut index = index;
    loop {
        index = if forward {
            index + 1
        } else {
            index.checked_sub(1)?
        };
        let sibling = siblings.get(index)?;
        match sibling {
            Node::Word(_) | Node::Source(_) => return None,
            Node::WhiteSpace(_) => {}
            _ => {
                let text = stringify_nlcst(sibling);
                return delimiters
                    .iter()
                    .find(|delimiter| **delimiter == text)
                    .copied();
            }
        }
    }
}
