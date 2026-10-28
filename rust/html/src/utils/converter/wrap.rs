use crate::utils::converter::store::{Id, Kind, Store};

// Splitting recurses once per nested link or delete; past the red zone the stack grows on
// the heap
const STACK_RED_ZONE: usize = 64 * 1024;
const STACK_GROWTH: usize = 1024 * 1024;

impl<'a> Store<'a> {
    /// Wrap runs of phrasing content in paragraphs, dropping runs of only whitespace, as
    /// hast-util-to-mdast's `wrap` does.
    pub(crate) fn wrap(&mut self, nodes: Vec<Id>) -> Vec<Id> {
        let flattened = self.flatten(nodes);
        let mut results = vec![];
        let mut queue = vec![];
        for node in flattened {
            if self.is_phrasing(node) {
                queue.push(node);
            } else {
                self.wrap_phrasing(std::mem::take(&mut queue), &mut results);
                results.push(node);
            }
        }
        self.wrap_phrasing(queue, &mut results);
        results
    }

    fn wrap_phrasing(&mut self, nodes: Vec<Id>, results: &mut Vec<Id>) {
        if nodes.is_empty() {
            return;
        }
        let blank = nodes.iter().all(|node| match &self.items[*node].kind {
            Kind::Text(text) => text.is_whitespace(),
            _ => false,
        });
        if !blank {
            let children = drop_surrounding_breaks(self, nodes);
            results.push(self.add(Kind::Paragraph, children, None));
        }
    }

    // A link or delete that holds flow content goes inside each flow node instead
    fn split(&mut self, node: Id) -> Vec<Id> {
        let children = self.take_children(node);
        let children = self.expand(&children).into_owned();
        let flattened = self.flatten(children);
        let mut results = vec![];
        let mut queue = vec![];
        for child in flattened {
            if self.is_phrasing(child) {
                queue.push(child);
                continue;
            }
            if !queue.is_empty() {
                let parent =
                    self.clone_without_children(node, std::mem::take(&mut queue));
                results.push(parent);
            }
            if self.items[child].kind.is_parent() {
                let grandchildren = self.take_children(child);
                let parent = self.clone_without_children(node, grandchildren);
                results.push(self.clone_without_children(child, vec![parent]));
            } else {
                results.push(child);
            }
        }
        if !queue.is_empty() {
            results.push(self.clone_without_children(node, queue));
        }
        results
    }

    fn flatten(&mut self, nodes: Vec<Id>) -> Vec<Id> {
        let mut flattened = Vec::with_capacity(nodes.len());
        for node in nodes {
            let item = &self.items[node];
            if matches!(item.kind, Kind::Delete | Kind::Link { .. }) && item.flow {
                let split = stacker::maybe_grow(STACK_RED_ZONE, STACK_GROWTH, || {
                    self.split(node)
                });
                flattened.extend(split);
            } else {
                flattened.push(node);
            }
        }
        flattened
    }
}

/// Drop breaks at the start and end of phrasing content.
pub(crate) fn drop_surrounding_breaks(store: &Store<'_>, mut nodes: Vec<Id>) -> Vec<Id> {
    let is_break = |node: &Id| matches!(store.items[*node].kind, Kind::Break);
    let start = nodes.iter().take_while(|node| is_break(node)).count();
    let end = nodes.len()
        - nodes[start..]
            .iter()
            .rev()
            .take_while(|node| is_break(node))
            .count();
    nodes.truncate(end);
    nodes.drain(..start);
    nodes
}
