use textlane_core::nlcst::Node;

/// Each node `test` accepts, in preorder, as unist-util-visit; with `skip`, never the nodes
/// inside one it accepted.
pub(crate) fn visit_nlcst<'t, 'a>(
    tree: &'t Node<'a>,
    test: impl Fn(&Node<'a>) -> bool,
    skip: bool,
    mut visitor: impl FnMut(&'t Node<'a>),
) {
    let mut stack = vec![tree];
    while let Some(node) = stack.pop() {
        let accepted = test(node);
        if accepted {
            visitor(node);
        }
        if !(accepted && skip)
            && let Some(children) = node.children()
        {
            stack.extend(children.iter().rev());
        }
    }
}
