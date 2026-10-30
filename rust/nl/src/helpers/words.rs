use textlane_core::nlcst::Node;

/// Where a word sits: its siblings, its index there and the index of the first and last word
/// among them.
pub(crate) struct WordPlace<'n, 'a> {
    pub siblings: &'n [Node<'a>],
    pub index: usize,
    pub first_word: usize,
    pub last_word: usize,
}

/// Call `visitor` on every word under `tree` in document order, as unist-util-visit does with
/// a `WordNode` test; the tree itself has no parent, so it is never visited.
pub(crate) fn visit_words<'n, 'a>(
    tree: &'n Node<'a>,
    mut visitor: impl FnMut(&'n Node<'a>, &WordPlace<'n, 'a>),
) {
    let mut frames: Vec<WordPlace<'n, 'a>> = open_frame(tree).into_iter().collect();
    while let Some(place) = frames.last_mut() {
        let Some(node) = place.siblings.get(place.index) else {
            frames.pop();
            continue;
        };
        if matches!(node, Node::Word(_)) {
            visitor(node, place);
        }
        place.index += 1;
        frames.extend(open_frame(node));
    }
}

// Each parent's word bounds are found once, so checks that look for words before or after a
// word stay linear in long sentences
fn open_frame<'n, 'a>(parent: &'n Node<'a>) -> Option<WordPlace<'n, 'a>> {
    let siblings: &'n [Node<'a>] = parent.children()?;
    let is_word = |node: &Node<'_>| matches!(node, Node::Word(_));
    Some(WordPlace {
        siblings,
        index: 0,
        first_word: siblings.iter().position(is_word).unwrap_or(usize::MAX),
        last_word: siblings.iter().rposition(is_word).unwrap_or(0),
    })
}
