use crate::helpers::foreign::is_integration_point;
use crate::helpers::tag::TagId;
use crate::models::namespace::Namespace;
use crate::models::parse_error::{ErrorCode, ParseError};
use crate::models::token::{CharacterToken, CommentToken, Span, TagToken, Token};
use crate::utils::builder::dom::{DocumentMode, Dom, Location, NodeData, NodeId};
use crate::utils::builder::formatting::{Entry, FormattingList};
use crate::utils::tokenizer::machine::{State, Tokenizer};

/// parse5's insertion modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoScript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

// The tag a pop closes an element with: parse5's `currentToken`, the last start or end tag
#[derive(Clone, Debug)]
pub(crate) struct Closing {
    pub(crate) end_tag_name: Option<String>,
    pub(crate) span: Span,
}

const TAG_COUNT: usize = TagId::Xmp as usize + 1;

/// parse5's tree builder over the tokenizer, building a `Dom`.
pub(crate) struct Parser<'s> {
    pub(crate) tokenizer: Tokenizer<'s>,
    pub(crate) dom: Dom,
    pub(crate) document: NodeId,
    pub(crate) fragment_context: Option<NodeId>,
    pub(crate) fragment_context_id: TagId,
    pub(crate) mode: Mode,
    pub(crate) original_mode: Mode,
    pub(crate) head_element: Option<NodeId>,
    pub(crate) form_element: Option<NodeId>,
    pub(crate) stack: Vec<NodeId>,
    pub(crate) stack_tags: Vec<TagId>,
    pub(crate) template_count: usize,
    // How many HTML elements of each tag are open, so a scope check for a tag that is not
    // open answers without walking a deep stack
    html_tag_counts: [u32; TAG_COUNT],
    tag_counts: [u32; TAG_COUNT],
    unknown_counts: std::collections::HashMap<String, u32>,
    on_stack: Vec<u32>,
    pub(crate) formatting: FormattingList,
    pub(crate) current_not_in_html: bool,
    // parse5's `tmplInsertionModeStack`, its front last
    pub(crate) template_modes: Vec<Mode>,
    pub(crate) pending_characters: Vec<CharacterToken>,
    pub(crate) has_non_whitespace_pending: bool,
    pub(crate) frameset_ok: bool,
    pub(crate) skip_next_new_line: bool,
    pub(crate) foster_parenting: bool,
    pub(crate) stopped: bool,
    pub(crate) closing: Option<Closing>,
    pub(crate) errors: Vec<ParseError>,
}

impl<'s> Parser<'s> {
    /// A parser for a whole document.
    pub(crate) fn for_document(input: &'s str) -> Parser<'s> {
        let mut dom = Dom::default();
        let document = dom.create(NodeData::Document {
            quirks: DocumentMode::NoQuirks,
        });
        let mut parser = Parser::new(input, dom, document, None);
        parser.set_context_modes(Some(document), None);
        parser
    }

    /// A parser for a fragment in a `template`, as hast-util-from-html parses fragments.
    pub(crate) fn for_fragment(input: &'s str) -> Parser<'s> {
        let mut dom = Dom::default();
        let context = dom.create_element("template", Namespace::Html, Vec::new());
        let document = dom.create_element("documentmock", Namespace::Html, Vec::new());
        let mut parser = Parser::new(input, dom, document, Some(context));
        parser.fragment_context_id = TagId::Template;
        parser.set_context_modes(Some(context), Some(TagId::Template));
        parser.template_modes.push(Mode::InTemplate);
        parser.insert_fake_root_element();
        parser.reset_insertion_mode();
        parser
    }

    fn new(
        input: &'s str,
        dom: Dom,
        document: NodeId,
        fragment_context: Option<NodeId>,
    ) -> Parser<'s> {
        Parser {
            tokenizer: Tokenizer::new(input),
            dom,
            document,
            fragment_context,
            fragment_context_id: TagId::Unknown,
            mode: Mode::Initial,
            original_mode: Mode::Initial,
            head_element: None,
            form_element: None,
            stack: Vec::new(),
            stack_tags: Vec::new(),
            template_count: 0,
            html_tag_counts: [0; TAG_COUNT],
            tag_counts: [0; TAG_COUNT],
            unknown_counts: std::collections::HashMap::new(),
            on_stack: Vec::new(),
            formatting: FormattingList::default(),
            current_not_in_html: false,
            template_modes: Vec::new(),
            pending_characters: Vec::new(),
            has_non_whitespace_pending: false,
            frameset_ok: true,
            skip_next_new_line: false,
            foster_parenting: false,
            stopped: false,
            closing: None,
            errors: Vec::new(),
        }
    }

    /// Run the tokenizer to the end, feeding each token to the tree builder.
    pub(crate) fn run(&mut self) {
        loop {
            let token = self.tokenizer.next_token();
            // Errors in the order parse5 reports them: the tokenizer's for a token before the
            // tree builder's while handling it
            self.errors.append(&mut self.tokenizer.errors);
            let is_eof = matches!(token, Token::Eof(_));
            self.dispatch(token);
            if is_eof || self.stopped {
                break;
            }
        }
    }

    /// The fragment's children, moved out of the fake `html` element into a fragment node.
    pub(crate) fn take_fragment(&mut self) -> NodeId {
        let fragment = self.dom.create(NodeData::Fragment);
        if let Some(root) = self.dom.nodes[self.document].first_child {
            self.dom.adopt_children(root, fragment);
        }
        fragment
    }

    pub(crate) fn err(&mut self, span: Span, code: ErrorCode, before_token: bool) {
        let end = if before_token { span.start } else { span.end };
        self.errors.push(ParseError {
            code,
            start: span.start,
            end,
        });
    }

    // Stack of open elements

    pub(crate) fn current(&self) -> Option<NodeId> {
        self.stack.last().copied()
    }

    pub(crate) fn current_tag(&self) -> Option<TagId> {
        self.stack_tags.last().copied()
    }

    fn is_in_template(&self) -> bool {
        self.current_tag() == Some(TagId::Template)
            && self
                .current()
                .is_some_and(|node| self.dom.namespace(node) == Namespace::Html)
    }

    /// Where content goes: the current node, its template content, or the document.
    pub(crate) fn current_parent(&self) -> NodeId {
        match self.current() {
            Some(node) if self.is_in_template() => {
                self.dom.template_content(node).unwrap_or(node)
            }
            Some(node) => node,
            None => self.document,
        }
    }

    fn mark_open(&mut self, node: NodeId, tag: TagId, delta: i32) {
        if self.on_stack.len() <= node {
            self.on_stack.resize(self.dom.nodes.len().max(node + 1), 0);
        }
        self.on_stack[node] = self.on_stack[node].saturating_add_signed(delta);
        if self.dom.namespace(node) == Namespace::Html {
            let count = &mut self.html_tag_counts[tag as usize];
            *count = count.saturating_add_signed(delta);
        }
        let count = &mut self.tag_counts[tag as usize];
        *count = count.saturating_add_signed(delta);
        if tag == TagId::Unknown {
            let name = self.dom.tag_name(node).to_string();
            let count = self.unknown_counts.entry(name).or_default();
            *count = count.saturating_add_signed(delta);
        }
    }

    /// Whether an element with this tag, in any namespace, is open.
    pub(crate) fn has_open_tag(&self, tag: TagId, name: &str) -> bool {
        if tag == TagId::Unknown {
            self.unknown_counts
                .get(name)
                .is_some_and(|count| *count > 0)
        } else {
            self.tag_counts[tag as usize] > 0
        }
    }

    pub(crate) fn push(&mut self, node: NodeId, tag: TagId) {
        self.stack.push(node);
        self.stack_tags.push(tag);
        self.mark_open(node, tag, 1);
        if self.is_in_template() {
            self.template_count += 1;
        }
        if self.stack.len() > 1 {
            self.set_context_modes(Some(node), Some(tag));
        }
    }

    pub(crate) fn pop(&mut self) {
        if self.template_count > 0 && self.is_in_template() {
            self.template_count -= 1;
        }
        let (Some(node), Some(tag)) = (self.stack.pop(), self.stack_tags.pop()) else {
            return;
        };
        self.mark_open(node, tag, -1);
        self.on_item_pop(node, true);
    }

    fn on_item_pop(&mut self, node: NodeId, is_top: bool) {
        if let Some(closing) = self.closing.clone() {
            self.set_end_location(node, &closing);
        }
        if is_top {
            if self.stack.len() == 1 && self.fragment_context.is_some() {
                self.set_context_modes(
                    self.fragment_context,
                    Some(self.fragment_context_id),
                );
            } else {
                self.set_context_modes(self.current(), self.current_tag());
            }
        }
    }

    pub(crate) fn set_context_modes(
        &mut self,
        current: Option<NodeId>,
        tag: Option<TagId>,
    ) {
        let is_html = current.is_some_and(|node| {
            node == self.document || self.dom.namespace(node) == Namespace::Html
        });
        self.current_not_in_html = !is_html;
        self.tokenizer.in_foreign_node = match (is_html, current, tag) {
            (false, Some(node), Some(tag)) => !self.is_integration_point(tag, node, None),
            _ => false,
        };
    }

    pub(crate) fn index_of(&self, node: NodeId) -> Option<usize> {
        if !self.contains(node) {
            return None;
        }
        self.stack.iter().rposition(|each| *each == node)
    }

    pub(crate) fn contains(&self, node: NodeId) -> bool {
        self.on_stack.get(node).is_some_and(|count| *count > 0)
    }

    pub(crate) fn replace_on_stack(&mut self, old: NodeId, new: NodeId) {
        if let Some(index) = self.index_of(old) {
            let tag = self.stack_tags[index];
            self.mark_open(old, tag, -1);
            self.stack[index] = new;
            self.mark_open(new, tag, 1);
        }
    }

    pub(crate) fn insert_after_on_stack(
        &mut self,
        reference: NodeId,
        node: NodeId,
        tag: TagId,
    ) {
        let index = self.index_of(reference).map_or(0, |index| index + 1);
        self.stack.insert(index, node);
        self.stack_tags.insert(index, tag);
        self.mark_open(node, tag, 1);
        if index + 1 == self.stack.len() {
            self.set_context_modes(Some(node), Some(tag));
        }
    }

    pub(crate) fn pop_until_tag_popped(&mut self, tag: TagId) {
        // The topmost HTML one; one at the bottom counts whatever its namespace
        let target = (0..self.stack.len()).rev().find(|index| {
            self.stack_tags[*index] == tag
                && (*index == 0
                    || self.dom.namespace(self.stack[*index]) == Namespace::Html)
        });
        self.shorten_to(target.unwrap_or(0));
    }

    pub(crate) fn shorten_to(&mut self, length: usize) {
        while self.stack.len() > length {
            if self.template_count > 0 && self.is_in_template() {
                self.template_count -= 1;
            }
            let (Some(node), Some(tag)) = (self.stack.pop(), self.stack_tags.pop())
            else {
                return;
            };
            self.mark_open(node, tag, -1);
            let is_top = self.stack.len() <= length;
            self.on_item_pop(node, is_top);
        }
    }

    fn index_of_tags(&self, tags: &[TagId], namespace: Namespace) -> Option<usize> {
        (0..self.stack.len()).rev().find(|index| {
            tags.contains(&self.stack_tags[*index])
                && self.dom.namespace(self.stack[*index]) == namespace
        })
    }

    pub(crate) fn pop_until_numbered_header_popped(&mut self) {
        let index = self.index_of_tags(&NUMBERED_HEADERS, Namespace::Html);
        self.shorten_to(index.unwrap_or(0));
    }

    pub(crate) fn pop_until_table_cell_popped(&mut self) {
        let index = self.index_of_tags(&[TagId::Td, TagId::Th], Namespace::Html);
        self.shorten_to(index.unwrap_or(0));
    }

    pub(crate) fn pop_all_up_to_html_element(&mut self) {
        self.template_count = 0;
        self.shorten_to(1);
    }

    fn clear_back_to(&mut self, tags: &[TagId]) {
        let index = self.index_of_tags(tags, Namespace::Html);
        self.shorten_to(index.map_or(0, |index| index + 1));
    }

    pub(crate) fn clear_back_to_table_context(&mut self) {
        self.clear_back_to(&[TagId::Table, TagId::Template, TagId::Html]);
    }

    pub(crate) fn clear_back_to_table_body_context(&mut self) {
        self.clear_back_to(&[
            TagId::Tbody,
            TagId::Tfoot,
            TagId::Thead,
            TagId::Template,
            TagId::Html,
        ]);
    }

    pub(crate) fn clear_back_to_table_row_context(&mut self) {
        self.clear_back_to(&[TagId::Tr, TagId::Template, TagId::Html]);
    }

    pub(crate) fn remove_from_stack(&mut self, node: NodeId) {
        let Some(index) = self.index_of(node) else {
            return;
        };
        if index + 1 == self.stack.len() {
            self.pop();
        } else {
            let tag = self.stack_tags.remove(index);
            self.stack.remove(index);
            self.mark_open(node, tag, -1);
            self.on_item_pop(node, false);
        }
    }

    pub(crate) fn properly_nested_body(&self) -> Option<NodeId> {
        (self.stack.len() >= 2 && self.stack_tags[1] == TagId::Body)
            .then(|| self.stack[1])
    }

    pub(crate) fn common_ancestor(&self, node: NodeId) -> Option<NodeId> {
        self.index_of(node)
            .and_then(|index| index.checked_sub(1))
            .map(|index| self.stack[index])
    }

    pub(crate) fn is_root_html_current(&self) -> bool {
        self.stack.len() == 1 && self.stack_tags[0] == TagId::Html
    }

    fn has_in_dynamic_scope(&self, tag: TagId, scope: &[TagId]) -> bool {
        // An HTML `html` element at the bottom ends every scope, so a tag that is not open
        // is not in scope
        if self.html_tag_counts[tag as usize] == 0
            && self
                .stack
                .first()
                .is_some_and(|node| self.dom.namespace(*node) == Namespace::Html)
            && self.stack_tags.first() == Some(&TagId::Html)
        {
            return false;
        }
        for index in (0..self.stack.len()).rev() {
            let current = self.stack_tags[index];
            match self.dom.namespace(self.stack[index]) {
                Namespace::Html => {
                    if current == tag {
                        return true;
                    }
                    if SCOPING_HTML.contains(&current) || scope.contains(&current) {
                        return false;
                    }
                }
                Namespace::Svg if SCOPING_SVG.contains(&current) => return false,
                Namespace::MathMl if SCOPING_MATHML.contains(&current) => return false,
                _ => {}
            }
        }
        true
    }

    pub(crate) fn has_in_scope(&self, tag: TagId) -> bool {
        self.has_in_dynamic_scope(tag, &[])
    }

    pub(crate) fn has_in_list_item_scope(&self, tag: TagId) -> bool {
        self.has_in_dynamic_scope(tag, &[TagId::Ol, TagId::Ul])
    }

    pub(crate) fn has_in_button_scope(&self, tag: TagId) -> bool {
        self.has_in_dynamic_scope(tag, &[TagId::Button])
    }

    pub(crate) fn has_numbered_header_in_scope(&self) -> bool {
        for index in (0..self.stack.len()).rev() {
            let current = self.stack_tags[index];
            match self.dom.namespace(self.stack[index]) {
                Namespace::Html => {
                    if NUMBERED_HEADERS.contains(&current) {
                        return true;
                    }
                    if SCOPING_HTML.contains(&current) {
                        return false;
                    }
                }
                Namespace::Svg if SCOPING_SVG.contains(&current) => return false,
                Namespace::MathMl if SCOPING_MATHML.contains(&current) => return false,
                _ => {}
            }
        }
        true
    }

    pub(crate) fn has_in_table_scope(&self, tag: TagId) -> bool {
        for index in (0..self.stack.len()).rev() {
            if self.dom.namespace(self.stack[index]) != Namespace::Html {
                continue;
            }
            let current = self.stack_tags[index];
            if current == tag {
                return true;
            }
            if matches!(current, TagId::Table | TagId::Html) {
                return false;
            }
        }
        true
    }

    pub(crate) fn has_table_body_context_in_table_scope(&self) -> bool {
        for index in (0..self.stack.len()).rev() {
            if self.dom.namespace(self.stack[index]) != Namespace::Html {
                continue;
            }
            match self.stack_tags[index] {
                TagId::Tbody | TagId::Thead | TagId::Tfoot => return true,
                TagId::Table | TagId::Html => return false,
                _ => {}
            }
        }
        true
    }

    pub(crate) fn has_in_select_scope(&self, tag: TagId) -> bool {
        for index in (0..self.stack.len()).rev() {
            if self.dom.namespace(self.stack[index]) != Namespace::Html {
                continue;
            }
            let current = self.stack_tags[index];
            if current == tag {
                return true;
            }
            if !matches!(current, TagId::Option | TagId::Optgroup) {
                return false;
            }
        }
        true
    }

    pub(crate) fn generate_implied_end_tags(&mut self) {
        while self
            .current_tag()
            .is_some_and(|tag| IMPLIED_END.contains(&tag))
        {
            self.pop();
        }
    }

    pub(crate) fn generate_implied_end_tags_thoroughly(&mut self) {
        while self.current_tag().is_some_and(|tag| {
            IMPLIED_END.contains(&tag) || IMPLIED_END_THOROUGH.contains(&tag)
        }) {
            self.pop();
        }
    }

    pub(crate) fn generate_implied_end_tags_except(&mut self, exclusion: TagId) {
        while self.current_tag().is_some_and(|tag| {
            tag != exclusion
                && (IMPLIED_END.contains(&tag) || IMPLIED_END_THOROUGH.contains(&tag))
        }) {
            self.pop();
        }
    }

    // Tree insertion

    pub(crate) fn is_integration_point(
        &self,
        tag: TagId,
        node: NodeId,
        foreign: Option<Namespace>,
    ) -> bool {
        let namespace = self.dom.namespace(node);
        let attributes: Vec<(&str, &str)> = self
            .dom
            .element(node)
            .map(|element| {
                element
                    .attrs
                    .iter()
                    .map(|attribute| (attribute.name.as_str(), attribute.value.as_str()))
                    .collect()
            })
            .unwrap_or_default();
        is_integration_point(tag, namespace, &attributes, foreign)
    }

    pub(crate) fn is_special(&self, node: NodeId, tag: TagId) -> bool {
        match self.dom.namespace(node) {
            Namespace::Html => SPECIAL_HTML.contains(&tag),
            Namespace::MathMl => SCOPING_MATHML.contains(&tag),
            Namespace::Svg => SCOPING_SVG.contains(&tag),
            _ => false,
        }
    }

    fn attach_element(&mut self, element: NodeId, span: Option<Span>) {
        if let Some(span) = span {
            let mut location = Location::of(span);
            location.start_tag = Some(span);
            self.dom.nodes[element].location = Some(location);
        }
        if self.should_foster_parent() {
            self.foster_parent_element(element);
        } else {
            let parent = self.current_parent();
            self.dom.append_child(parent, element);
        }
    }

    pub(crate) fn append_element(&mut self, token: &TagToken, namespace: Namespace) {
        let element =
            self.dom
                .create_element(&token.tag_name, namespace, token.attrs.clone());
        self.attach_element(element, Some(token.span));
    }

    pub(crate) fn insert_element(&mut self, token: &TagToken, namespace: Namespace) {
        let element =
            self.dom
                .create_element(&token.tag_name, namespace, token.attrs.clone());
        self.attach_element(element, Some(token.span));
        self.push(element, token.tag_id);
    }

    pub(crate) fn insert_fake_element(&mut self, tag_name: &str, tag: TagId) {
        let element = self
            .dom
            .create_element(tag_name, Namespace::Html, Vec::new());
        self.attach_element(element, None);
        self.push(element, tag);
    }

    pub(crate) fn insert_template(&mut self, token: &TagToken) {
        let element = self.dom.create_element(
            &token.tag_name,
            Namespace::Html,
            token.attrs.clone(),
        );
        let content = self.dom.create(NodeData::Fragment);
        if let Some(element) = self.dom.element_mut(element) {
            element.template_content = Some(content);
        }
        self.attach_element(element, Some(token.span));
        self.push(element, token.tag_id);
    }

    pub(crate) fn insert_fake_root_element(&mut self) {
        let element = self.dom.create_element("html", Namespace::Html, Vec::new());
        let parent = self.current().unwrap_or(self.document);
        self.dom.append_child(parent, element);
        self.push(element, TagId::Html);
    }

    pub(crate) fn append_comment(&mut self, token: &CommentToken, parent: NodeId) {
        let node = self.dom.create(NodeData::Comment(token.data.clone()));
        self.dom.append_child(parent, node);
        self.dom.nodes[node].location = Some(Location::of(token.span));
    }

    pub(crate) fn insert_characters(&mut self, token: &CharacterToken) {
        let node = if self.should_foster_parent() {
            let (parent, before) = self.find_foster_parenting_location();
            self.dom.insert_text(parent, &token.chars, before)
        } else {
            let parent = self.current_parent();
            self.dom.insert_text(parent, &token.chars, None)
        };
        match &mut self.dom.nodes[node].location {
            Some(location) => location.end = token.span.end,
            None => self.dom.nodes[node].location = Some(Location::of(token.span)),
        }
    }

    pub(crate) fn set_end_location(&mut self, element: NodeId, closing: &Closing) {
        let matches = closing
            .end_tag_name
            .as_deref()
            .is_some_and(|name| name == self.dom.tag_name(element));
        let Some(location) = &mut self.dom.nodes[element].location else {
            return;
        };
        if matches {
            location.end_tag = Some(closing.span);
            location.end = closing.span.end;
        } else {
            location.end = closing.span.start;
        }
    }

    pub(crate) fn set_document_type(&mut self, span: Span) {
        let existing = self
            .dom
            .children(self.document)
            .find(|node| matches!(self.dom.nodes[*node].data, NodeData::Doctype));
        let node = match existing {
            Some(node) => node,
            None => {
                let node = self.dom.create(NodeData::Doctype);
                self.dom.append_child(self.document, node);
                node
            }
        };
        self.dom.nodes[node].location = Some(Location::of(span));
    }

    pub(crate) fn set_document_mode(&mut self, mode: DocumentMode) {
        if let NodeData::Document { quirks } = &mut self.dom.nodes[self.document].data {
            *quirks = mode;
        }
    }

    pub(crate) fn document_mode(&self) -> DocumentMode {
        match self.dom.nodes[self.document].data {
            NodeData::Document { quirks } => quirks,
            _ => DocumentMode::NoQuirks,
        }
    }

    pub(crate) fn switch_to_text_parsing(&mut self, token: &TagToken, state: State) {
        self.insert_element(token, Namespace::Html);
        self.tokenizer.state = state;
        self.original_mode = self.mode;
        self.mode = Mode::Text;
    }

    pub(crate) fn adjusted_current(&self) -> Option<NodeId> {
        if self.stack.len() == 1 && self.fragment_context.is_some() {
            self.fragment_context
        } else {
            self.current()
        }
    }

    pub(crate) fn reconstruct_active_formatting_elements(&mut self) {
        let entries = &self.formatting.entries;
        if entries.is_empty() {
            return;
        }
        // From the newest entry back to the first marker or open element
        let mut first_unopened = entries.len();
        for (index, entry) in entries.iter().enumerate().rev() {
            match entry {
                Entry::Marker => break,
                Entry::Element(entry) if self.contains(entry.element) => break,
                Entry::Element(_) => first_unopened = index,
            }
        }
        for index in first_unopened..self.formatting.entries.len() {
            let Entry::Element(entry) = &self.formatting.entries[index] else {
                continue;
            };
            let (id, token, element) = (entry.id, entry.token.clone(), entry.element);
            let namespace = self.dom.namespace(element);
            self.insert_element(&token, namespace);
            if let Some(current) = self.current() {
                self.formatting.set_element(id, current);
            }
        }
    }

    pub(crate) fn close_table_cell(&mut self) {
        self.generate_implied_end_tags();
        self.pop_until_table_cell_popped();
        self.formatting.clear_to_last_marker();
        self.mode = Mode::InRow;
    }

    pub(crate) fn close_p_element(&mut self) {
        self.generate_implied_end_tags_except(TagId::P);
        self.pop_until_tag_popped(TagId::P);
    }

    pub(crate) fn reset_insertion_mode(&mut self) {
        for index in (0..self.stack.len()).rev() {
            let tag = if index == 0 && self.fragment_context.is_some() {
                self.fragment_context_id
            } else {
                self.stack_tags[index]
            };
            self.mode = match tag {
                TagId::Tr => Mode::InRow,
                TagId::Tbody | TagId::Thead | TagId::Tfoot => Mode::InTableBody,
                TagId::Caption => Mode::InCaption,
                TagId::Colgroup => Mode::InColumnGroup,
                TagId::Table => Mode::InTable,
                TagId::Body => Mode::InBody,
                TagId::Frameset => Mode::InFrameset,
                TagId::Select => {
                    self.reset_insertion_mode_for_select(index);
                    return;
                }
                TagId::Template => {
                    self.template_modes.last().copied().unwrap_or(Mode::InBody)
                }
                TagId::Html => {
                    if self.head_element.is_some() {
                        Mode::AfterHead
                    } else {
                        Mode::BeforeHead
                    }
                }
                TagId::Td | TagId::Th if index > 0 => Mode::InCell,
                TagId::Head if index > 0 => Mode::InHead,
                _ => continue,
            };
            return;
        }
        self.mode = Mode::InBody;
    }

    fn reset_insertion_mode_for_select(&mut self, select: usize) {
        for index in (1..select).rev() {
            match self.stack_tags[index] {
                TagId::Template => break,
                TagId::Table => {
                    self.mode = Mode::InSelectInTable;
                    return;
                }
                _ => {}
            }
        }
        self.mode = Mode::InSelect;
    }

    pub(crate) fn should_foster_parent(&self) -> bool {
        self.foster_parenting
            && self
                .current_tag()
                .is_some_and(|tag| TABLE_STRUCTURE.contains(&tag))
    }

    pub(crate) fn find_foster_parenting_location(&self) -> (NodeId, Option<NodeId>) {
        for index in (0..self.stack.len()).rev() {
            let node = self.stack[index];
            match self.stack_tags[index] {
                TagId::Template if self.dom.namespace(node) == Namespace::Html => {
                    return (self.dom.template_content(node).unwrap_or(node), None);
                }
                TagId::Table => {
                    return match self.dom.nodes[node].parent {
                        Some(parent) => (parent, Some(node)),
                        None => (self.stack[index.saturating_sub(1)], None),
                    };
                }
                _ => {}
            }
        }
        (self.stack.first().copied().unwrap_or(self.document), None)
    }

    pub(crate) fn foster_parent_element(&mut self, element: NodeId) {
        let (parent, before) = self.find_foster_parenting_location();
        match before {
            Some(before) => self.dom.insert_before(parent, element, before),
            None => self.dom.append_child(parent, element),
        }
    }
}

pub(crate) const NUMBERED_HEADERS: [TagId; 6] = [
    TagId::H1,
    TagId::H2,
    TagId::H3,
    TagId::H4,
    TagId::H5,
    TagId::H6,
];

pub(crate) const TABLE_STRUCTURE: [TagId; 5] = [
    TagId::Table,
    TagId::Tbody,
    TagId::Tfoot,
    TagId::Thead,
    TagId::Tr,
];

const IMPLIED_END: [TagId; 10] = [
    TagId::Dd,
    TagId::Dt,
    TagId::Li,
    TagId::Optgroup,
    TagId::Option,
    TagId::P,
    TagId::Rb,
    TagId::Rp,
    TagId::Rt,
    TagId::Rtc,
];

const IMPLIED_END_THOROUGH: [TagId; 8] = [
    TagId::Caption,
    TagId::Colgroup,
    TagId::Tbody,
    TagId::Td,
    TagId::Tfoot,
    TagId::Th,
    TagId::Thead,
    TagId::Tr,
];

const SCOPING_HTML: [TagId; 9] = [
    TagId::Applet,
    TagId::Caption,
    TagId::Html,
    TagId::Marquee,
    TagId::Object,
    TagId::Table,
    TagId::Td,
    TagId::Template,
    TagId::Th,
];

const SCOPING_MATHML: [TagId; 6] = [
    TagId::AnnotationXml,
    TagId::Mi,
    TagId::Mn,
    TagId::Mo,
    TagId::Ms,
    TagId::Mtext,
];

const SCOPING_SVG: [TagId; 3] = [TagId::Desc, TagId::ForeignObject, TagId::Title];

const SPECIAL_HTML: [TagId; 81] = [
    TagId::Address,
    TagId::Applet,
    TagId::Area,
    TagId::Article,
    TagId::Aside,
    TagId::Base,
    TagId::Basefont,
    TagId::Bgsound,
    TagId::Blockquote,
    TagId::Body,
    TagId::Br,
    TagId::Button,
    TagId::Caption,
    TagId::Center,
    TagId::Col,
    TagId::Colgroup,
    TagId::Dd,
    TagId::Details,
    TagId::Dir,
    TagId::Div,
    TagId::Dl,
    TagId::Dt,
    TagId::Embed,
    TagId::Fieldset,
    TagId::Figcaption,
    TagId::Figure,
    TagId::Footer,
    TagId::Form,
    TagId::Frame,
    TagId::Frameset,
    TagId::H1,
    TagId::H2,
    TagId::H3,
    TagId::H4,
    TagId::H5,
    TagId::H6,
    TagId::Head,
    TagId::Header,
    TagId::Hgroup,
    TagId::Hr,
    TagId::Html,
    TagId::Iframe,
    TagId::Img,
    TagId::Input,
    TagId::Li,
    TagId::Link,
    TagId::Listing,
    TagId::Main,
    TagId::Marquee,
    TagId::Menu,
    TagId::Meta,
    TagId::Nav,
    TagId::Noembed,
    TagId::Noframes,
    TagId::Noscript,
    TagId::Object,
    TagId::Ol,
    TagId::P,
    TagId::Param,
    TagId::Plaintext,
    TagId::Pre,
    TagId::Script,
    TagId::Section,
    TagId::Select,
    TagId::Source,
    TagId::Style,
    TagId::Summary,
    TagId::Table,
    TagId::Tbody,
    TagId::Td,
    TagId::Template,
    TagId::Textarea,
    TagId::Tfoot,
    TagId::Th,
    TagId::Thead,
    TagId::Title,
    TagId::Tr,
    TagId::Track,
    TagId::Ul,
    TagId::Wbr,
    TagId::Xmp,
];
