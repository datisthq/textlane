use std::collections::HashMap;

use crate::models::token::TagToken;
use crate::utils::builder::dom::{Dom, NodeId};

// More identical formatting elements than this are dropped from the oldest ("Noah's Ark")
const NOAH_ARK_CAPACITY: usize = 3;

/// An entry of the list of active formatting elements.
#[derive(Clone, Debug)]
pub(crate) enum Entry {
    Marker,
    Element(ElementEntry),
}

#[derive(Clone, Debug)]
pub(crate) struct ElementEntry {
    pub(crate) id: usize,
    pub(crate) element: NodeId,
    pub(crate) token: TagToken,
    // How many markers are before it, and what makes it identical to another entry
    segment: usize,
    identity: String,
}

// Entry counts since each marker, so lookups that would scan the whole list for nothing
// answer at once: scanning a long list per tag is quadratic on hostile input
#[derive(Debug, Default)]
struct Counts {
    names: HashMap<String, usize>,
    identities: HashMap<String, usize>,
}

/// parse5's `FormattingElementList`, stored oldest first where parse5 keeps newest first:
/// parse5 inserts at the front, which is quadratic in a long list.
#[derive(Debug, Default)]
pub(crate) struct FormattingList {
    pub(crate) entries: Vec<Entry>,
    pub(crate) bookmark: Option<usize>,
    next_id: usize,
    counts: Vec<Counts>,
}

impl FormattingList {
    pub(crate) fn insert_marker(&mut self) {
        // The counts before any marker are the first
        if self.counts.is_empty() {
            self.counts.push(Counts::default());
        }
        self.entries.push(Entry::Marker);
        self.counts.push(Counts::default());
    }

    pub(crate) fn push_element(&mut self, dom: &Dom, element: NodeId, token: TagToken) {
        let identity = create_identity(dom, element);
        if self.count(&identity) >= NOAH_ARK_CAPACITY {
            self.ensure_noah_ark(dom, element);
        }
        let entry = self.create_entry(element, token, identity, self.segment());
        self.entries.push(Entry::Element(entry));
    }

    fn segment(&self) -> usize {
        self.counts.len().saturating_sub(1)
    }

    fn count(&self, identity: &str) -> usize {
        self.counts
            .last()
            .and_then(|counts| counts.identities.get(identity))
            .copied()
            .unwrap_or(0)
    }

    fn create_entry(
        &mut self,
        element: NodeId,
        token: TagToken,
        identity: String,
        segment: usize,
    ) -> ElementEntry {
        if self.counts.is_empty() {
            self.counts.push(Counts::default());
        }
        let last = self.counts.len() - 1;
        let counts = &mut self.counts[segment.min(last)];
        *counts.names.entry(token.tag_name.clone()).or_default() += 1;
        *counts.identities.entry(identity.clone()).or_default() += 1;
        let id = self.create_id();
        ElementEntry {
            id,
            element,
            token,
            segment,
            identity,
        }
    }

    fn forget(&mut self, entry: &Entry) {
        let Entry::Element(entry) = entry else {
            return;
        };
        if let Some(counts) = self.counts.get_mut(entry.segment) {
            if let Some(count) = counts.names.get_mut(&entry.token.tag_name) {
                *count = count.saturating_sub(1);
            }
            if let Some(count) = counts.identities.get_mut(&entry.identity) {
                *count = count.saturating_sub(1);
            }
        }
    }

    // At most three entries with the same tag name, namespace and attributes since the
    // last marker; the oldest beyond that is dropped
    fn ensure_noah_ark(&mut self, dom: &Dom, element: NodeId) {
        if self.entries.len() < NOAH_ARK_CAPACITY {
            return;
        }
        let Some(new) = dom.element(element) else {
            return;
        };
        let mut candidates = Vec::new();
        for (index, entry) in self.entries.iter().enumerate().rev() {
            let Entry::Element(entry) = entry else {
                break;
            };
            if let Some(old) = dom.element(entry.element)
                && old.tag_name == new.tag_name
                && old.namespace == new.namespace
                && old.attrs.len() == new.attrs.len()
            {
                candidates.push((index, entry.element));
            }
        }
        if candidates.len() < NOAH_ARK_CAPACITY {
            return;
        }
        // The list never holds more than three identical entries, so only the third one
        // found, the oldest, can be dropped
        let mut valid = 0;
        for (index, candidate) in candidates {
            let Some(old) = dom.element(candidate) else {
                continue;
            };
            let matches = old.attrs.iter().all(|old| {
                new.attrs
                    .iter()
                    .rev()
                    .find(|each| each.name == old.name)
                    .map(|each| &each.value)
                    == Some(&old.value)
            });
            if matches {
                valid += 1;
                if valid >= NOAH_ARK_CAPACITY {
                    let entry = self.entries.remove(index);
                    self.forget(&entry);
                    return;
                }
            }
        }
    }

    /// Insert where the bookmark is, newer than it.
    pub(crate) fn insert_element_after_bookmark(
        &mut self,
        dom: &Dom,
        element: NodeId,
        token: TagToken,
    ) {
        let identity = create_identity(dom, element);
        let position = self.bookmark.and_then(|bookmark| self.position(bookmark));
        let segment = match position.map(|index| &self.entries[index]) {
            Some(Entry::Element(entry)) => entry.segment,
            _ => self.segment(),
        };
        let entry = Entry::Element(self.create_entry(element, token, identity, segment));
        match position {
            Some(index) => self.entries.insert(index + 1, entry),
            None => self.entries.push(entry),
        }
    }

    pub(crate) fn remove_entry(&mut self, id: usize) {
        if let Some(index) = self.position(id) {
            let entry = self.entries.remove(index);
            self.forget(&entry);
        }
    }

    pub(crate) fn clear_to_last_marker(&mut self) {
        while let Some(entry) = self.entries.pop() {
            if matches!(entry, Entry::Marker) {
                self.counts.pop();
                break;
            }
            self.forget(&entry);
        }
    }

    /// The newest entry named `tag_name` since the last marker.
    pub(crate) fn find_in_scope(
        &self,
        dom: &Dom,
        tag_name: &str,
    ) -> Option<ElementEntry> {
        let count = self
            .counts
            .last()
            .and_then(|counts| counts.names.get(tag_name));
        if count.is_none_or(|count| *count == 0) {
            return None;
        }
        for entry in self.entries.iter().rev() {
            match entry {
                Entry::Marker => return None,
                Entry::Element(entry) if dom.tag_name(entry.element) == tag_name => {
                    return Some(entry.clone());
                }
                Entry::Element(_) => {}
            }
        }
        None
    }

    pub(crate) fn find_by_element(&self, element: NodeId) -> Option<ElementEntry> {
        self.entries.iter().rev().find_map(|entry| match entry {
            Entry::Element(entry) if entry.element == element => Some(entry.clone()),
            _ => None,
        })
    }

    pub(crate) fn find_by_id(&self, id: usize) -> Option<ElementEntry> {
        self.position(id)
            .and_then(|index| match &self.entries[index] {
                Entry::Element(entry) => Some(entry.clone()),
                Entry::Marker => None,
            })
    }

    pub(crate) fn set_element(&mut self, id: usize, element: NodeId) {
        if let Some(index) = self.position(id)
            && let Entry::Element(entry) = &mut self.entries[index]
        {
            entry.element = element;
        }
    }

    fn position(&self, id: usize) -> Option<usize> {
        self.entries
            .iter()
            .rposition(|entry| matches!(entry, Entry::Element(entry) if entry.id == id))
    }

    fn create_id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id
    }
}

// Entries are identical when their elements have the same tag name, namespace and
// attributes; the tokenizer drops duplicate attributes, so a sorted list compares them
fn create_identity(dom: &Dom, element: NodeId) -> String {
    let Some(element) = dom.element(element) else {
        return String::new();
    };
    let mut attributes: Vec<(&str, &str)> = element
        .attrs
        .iter()
        .map(|attribute| (attribute.name.as_str(), attribute.value.as_str()))
        .collect();
    attributes.sort_unstable();
    let mut identity = format!("{}\0{:?}", element.tag_name, element.namespace);
    for (name, value) in attributes {
        identity.push('\0');
        identity.push_str(name);
        identity.push('=');
        identity.push_str(value);
    }
    identity
}
