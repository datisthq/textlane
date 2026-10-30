use textlane_core::nlcst::Node;

use crate::actions::nlcst::stringify::stringify_nlcst;
use crate::helpers::literal::is_literal;
use crate::helpers::normalize::normalize_word;
use crate::models::phrases::Phrases;

/// A phrase found among a parent's children: `siblings[start..end]`.
pub(crate) struct Found<'t, 'a> {
    pub siblings: &'t [Node<'a>],
    /// Tells parents apart: the same for every match in one parent.
    pub parent: usize,
    pub start: usize,
    pub end: usize,
    pub phrase: usize,
}

struct Frame<'t, 'a> {
    children: &'t [Node<'a>],
    id: usize,
    next: usize,
    words: Option<(usize, usize)>,
    normalized: Vec<Option<String>>,
}

/// Every phrase found in `tree`, word by word in preorder, as nlcst-search does: later words
/// may follow over white space, and words used as literals start none.
pub(crate) fn search_nlcst<'t, 'a>(
    tree: &'t Node<'a>,
    phrases: &Phrases,
    mut handle: impl FnMut(Found<'t, 'a>),
) {
    let mut ids = 0;
    let mut stack = Vec::new();
    if let Some(children) = tree.children() {
        stack.push(create_frame(children, ids, phrases.allow_apostrophes));
    }
    while let Some(top) = stack.len().checked_sub(1) {
        let frame = &mut stack[top];
        let index = frame.next;
        let Some(child) = frame.children.get(index) else {
            stack.pop();
            continue;
        };
        frame.next += 1;
        let frame = &stack[top];
        if let (Node::Word(_), Some(word)) = (child, &frame.normalized[index])
            && !is_literal(frame.children, index, frame.words)
        {
            let starting = phrases.by_word.get(word).map_or(&[][..], Vec::as_slice);
            let doubled = if word == "*" { &phrases.any[..] } else { &[] };
            for &phrase in phrases.any.iter().chain(doubled).chain(starting) {
                if let Some(end) = match_phrase(frame, index, &phrases.rest[phrase]) {
                    handle(Found {
                        siblings: frame.children,
                        parent: frame.id,
                        start: index,
                        end,
                        phrase,
                    });
                }
            }
        }
        if let Some(children) = child.children().filter(|children| !children.is_empty()) {
            ids += 1;
            stack.push(create_frame(children, ids, phrases.allow_apostrophes));
        }
    }
}

// Each word normalized once per parent, however many phrases test it
fn create_frame<'t, 'a>(
    children: &'t [Node<'a>],
    id: usize,
    allow_apostrophes: bool,
) -> Frame<'t, 'a> {
    let normalized: Vec<Option<String>> = children
        .iter()
        .map(|child| {
            matches!(child, Node::Word(_))
                .then(|| normalize_word(&stringify_nlcst(child), allow_apostrophes))
        })
        .collect();
    let first = normalized.iter().position(Option::is_some);
    let last = normalized.iter().rposition(Option::is_some);
    Frame {
        children,
        id,
        next: 0,
        words: first.zip(last),
        normalized,
    }
}

fn match_phrase(
    frame: &Frame<'_, '_>,
    start: usize,
    rest: &[Option<String>],
) -> Option<usize> {
    let mut position = start + 1;
    for expected in rest {
        while matches!(frame.children.get(position), Some(Node::WhiteSpace(_))) {
            position += 1;
        }
        let word = frame.normalized.get(position)?.as_ref()?;
        if expected.as_ref().is_some_and(|expected| expected != word) {
            return None;
        }
        position += 1;
    }
    Some(position)
}
