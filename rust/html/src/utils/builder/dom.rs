use crate::models::namespace::Namespace;
use crate::models::token::{Attribute, Span};

/// An index into the `Dom`.
pub(crate) type NodeId = usize;

/// The mutable tree parse5's default tree adapter builds, kept in one arena with sibling
/// links: the tree builder moves nodes around (adoption agency, foster parenting), and
/// finding a node in its parent's child list each time would be quadratic.
#[derive(Debug, Default)]
pub(crate) struct Dom {
    pub(crate) nodes: Vec<DomNode>,
}

#[derive(Debug)]
pub(crate) struct DomNode {
    pub(crate) data: NodeData,
    pub(crate) parent: Option<NodeId>,
    pub(crate) first_child: Option<NodeId>,
    pub(crate) last_child: Option<NodeId>,
    pub(crate) previous: Option<NodeId>,
    pub(crate) next: Option<NodeId>,
    pub(crate) location: Option<Location>,
}

#[derive(Debug)]
pub(crate) enum NodeData {
    Document { quirks: DocumentMode },
    Fragment,
    Element(Element),
    Text(String),
    Comment(String),
    Doctype,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DocumentMode {
    NoQuirks,
    Quirks,
    LimitedQuirks,
}

#[derive(Debug)]
pub(crate) struct Element {
    pub(crate) tag_name: String,
    pub(crate) namespace: Namespace,
    pub(crate) attrs: Vec<Attribute>,
    pub(crate) template_content: Option<NodeId>,
}

/// parse5's `sourceCodeLocation`, in bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Location {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) start_tag: Option<Span>,
    pub(crate) end_tag: Option<Span>,
}

impl Location {
    pub(crate) fn of(span: Span) -> Location {
        Location {
            start: span.start,
            end: span.end,
            start_tag: None,
            end_tag: None,
        }
    }
}

impl Dom {
    pub(crate) fn create(&mut self, data: NodeData) -> NodeId {
        self.nodes.push(DomNode {
            data,
            parent: None,
            first_child: None,
            last_child: None,
            previous: None,
            next: None,
            location: None,
        });
        self.nodes.len() - 1
    }

    pub(crate) fn create_element(
        &mut self,
        tag_name: &str,
        namespace: Namespace,
        attrs: Vec<Attribute>,
    ) -> NodeId {
        self.create(NodeData::Element(Element {
            tag_name: tag_name.to_string(),
            namespace,
            attrs,
            template_content: None,
        }))
    }

    pub(crate) fn element(&self, node: NodeId) -> Option<&Element> {
        match &self.nodes[node].data {
            NodeData::Element(element) => Some(element),
            _ => None,
        }
    }

    pub(crate) fn element_mut(&mut self, node: NodeId) -> Option<&mut Element> {
        match &mut self.nodes[node].data {
            NodeData::Element(element) => Some(element),
            _ => None,
        }
    }

    /// The namespace of an element; the document counts as HTML.
    pub(crate) fn namespace(&self, node: NodeId) -> Namespace {
        self.element(node)
            .map_or(Namespace::Html, |element| element.namespace)
    }

    pub(crate) fn tag_name(&self, node: NodeId) -> &str {
        self.element(node)
            .map_or("", |element| element.tag_name.as_str())
    }

    pub(crate) fn template_content(&self, node: NodeId) -> Option<NodeId> {
        self.element(node)
            .and_then(|element| element.template_content)
    }

    pub(crate) fn append_child(&mut self, parent: NodeId, child: NodeId) {
        let last = self.nodes[parent].last_child;
        self.nodes[child].parent = Some(parent);
        self.nodes[child].previous = last;
        self.nodes[child].next = None;
        match last {
            Some(last) => self.nodes[last].next = Some(child),
            None => self.nodes[parent].first_child = Some(child),
        }
        self.nodes[parent].last_child = Some(child);
    }

    pub(crate) fn insert_before(
        &mut self,
        parent: NodeId,
        child: NodeId,
        reference: NodeId,
    ) {
        let previous = self.nodes[reference].previous;
        self.nodes[child].parent = Some(parent);
        self.nodes[child].previous = previous;
        self.nodes[child].next = Some(reference);
        self.nodes[reference].previous = Some(child);
        match previous {
            Some(previous) => self.nodes[previous].next = Some(child),
            None => self.nodes[parent].first_child = Some(child),
        }
    }

    pub(crate) fn detach(&mut self, node: NodeId) {
        let Some(parent) = self.nodes[node].parent else {
            return;
        };
        let previous = self.nodes[node].previous;
        let next = self.nodes[node].next;
        match previous {
            Some(previous) => self.nodes[previous].next = next,
            None => self.nodes[parent].first_child = next,
        }
        match next {
            Some(next) => self.nodes[next].previous = previous,
            None => self.nodes[parent].last_child = previous,
        }
        self.nodes[node].parent = None;
        self.nodes[node].previous = None;
        self.nodes[node].next = None;
    }

    /// Append text, merging it into a text node right before; returns the text node.
    pub(crate) fn insert_text(
        &mut self,
        parent: NodeId,
        text: &str,
        before: Option<NodeId>,
    ) -> NodeId {
        let previous = match before {
            Some(before) => self.nodes[before].previous,
            None => self.nodes[parent].last_child,
        };
        if let Some(previous) = previous
            && let NodeData::Text(value) = &mut self.nodes[previous].data
        {
            value.push_str(text);
            return previous;
        }
        let node = self.create(NodeData::Text(text.to_string()));
        match before {
            Some(before) => self.insert_before(parent, node, before),
            None => self.append_child(parent, node),
        }
        node
    }

    /// Move every child of `donor` to the end of `recipient`.
    pub(crate) fn adopt_children(&mut self, donor: NodeId, recipient: NodeId) {
        while let Some(child) = self.nodes[donor].first_child {
            self.detach(child);
            self.append_child(recipient, child);
        }
    }

    pub(crate) fn children(&self, node: NodeId) -> Children<'_> {
        Children {
            dom: self,
            next: self.nodes[node].first_child,
        }
    }
}

pub(crate) struct Children<'d> {
    dom: &'d Dom,
    next: Option<NodeId>,
}

impl Iterator for Children<'_> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let current = self.next?;
        self.next = self.dom.nodes[current].next;
        Some(current)
    }
}
