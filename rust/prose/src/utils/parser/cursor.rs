use std::collections::VecDeque;

use crate::utils::parser::store::Store;

/// A parent's children split at the node being visited: unist-util-modify-children splices
/// next to it, and a gap there makes each splice cost nothing.
pub(crate) struct Cursor {
    before: Vec<usize>,
    // Reversed: its last item is the current node
    after: Vec<usize>,
}

impl Cursor {
    pub(crate) fn new(children: VecDeque<usize>) -> Self {
        Self {
            before: Vec::new(),
            after: children.into_iter().rev().collect(),
        }
    }

    pub(crate) fn finish(mut self) -> VecDeque<usize> {
        let mut children = VecDeque::from(self.before);
        while let Some(id) = self.after.pop() {
            children.push_back(id);
        }
        children
    }

    /// The index of the current node.
    pub(crate) fn index(&self) -> usize {
        self.before.len()
    }

    pub(crate) fn len(&self) -> usize {
        self.before.len() + self.after.len()
    }

    /// The node at `index`.
    pub(crate) fn get(&self, index: usize) -> Option<usize> {
        match index.checked_sub(self.before.len()) {
            None => self.before.get(index).copied(),
            Some(ahead) => self
                .after
                .len()
                .checked_sub(ahead + 1)
                .map(|position| self.after[position]),
        }
    }

    /// Remove the node at `index`, which is the current one or after it.
    pub(crate) fn remove(&mut self, index: usize) {
        if let Some(position) = index
            .checked_sub(self.before.len())
            .and_then(|ahead| self.after.len().checked_sub(ahead + 1))
        {
            self.after.remove(position);
        }
    }

    /// Insert a node at `index`, the current one's or the next.
    pub(crate) fn insert(&mut self, index: usize, id: usize) {
        if index <= self.before.len() {
            self.before.insert(index, id);
        } else {
            let ahead = index - self.before.len();
            self.after
                .insert(self.after.len().saturating_sub(ahead), id);
        }
    }

    /// Make `index` the current node; returns whether a node is there.
    pub(crate) fn seek(&mut self, index: usize) -> bool {
        while self.before.len() > index {
            if let Some(id) = self.before.pop() {
                self.after.push(id);
            }
        }
        while self.before.len() < index {
            match self.after.pop() {
                Some(id) => self.before.push(id),
                None => return false,
            }
        }
        !self.after.is_empty()
    }
}

/// What a visitor wants next, as unist-util-modify-children reads its return value.
pub(crate) enum Next {
    /// The node after the visited one's index.
    Advance,
    /// The node at this index; a negative one restarts at the first.
    To(isize),
}

/// Visit the children of `parent` in order, as unist-util-modify-children does, letting the
/// visitor splice them through the cursor.
pub(crate) fn modify_children<'a>(
    store: &mut Store<'a>,
    parent: usize,
    mut visit: impl FnMut(&mut Store<'a>, &mut Cursor, usize) -> Next,
) {
    let mut cursor = Cursor::new(std::mem::take(store.children_mut(parent)));
    while cursor.seek(cursor.index()) {
        let index = cursor.index();
        let next = match visit(store, &mut cursor, index) {
            Next::Advance => index + 1,
            Next::To(index) => index.max(0) as usize,
        };
        if !cursor.seek(next) {
            break;
        }
    }
    *store.children_mut(parent) = cursor.finish();
}
