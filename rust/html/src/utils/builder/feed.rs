use textlane_core::hast::{Element, PropertyItem, PropertyValue};

use crate::actions::hast::stringify::{VOID_ELEMENTS, write_js_number};
use crate::helpers::property::find_property;
use crate::helpers::schema::{BOOLEAN, COMMA_SEPARATED, HTML, SVG};
use crate::helpers::tag::find_tag_id;
use crate::helpers::value::is_truthy;
use crate::models::namespace::Namespace;
use crate::models::token::{
    Attribute, CharacterKind, CharacterToken, CommentToken, DoctypeToken, Span, TagToken,
    Token,
};
use crate::utils::builder::dom::{NodeData, NodeId};
use crate::utils::builder::parser::Parser;
use crate::utils::tokenizer::machine::State;

// hast-util-raw's handlers: hast nodes become tokens handed straight to the tree builder,
// raw nodes are written to the tokenizer as chunks, and the tokenizer is reset in between
impl<'s> Parser<'s> {
    pub(crate) fn feed_raw(&mut self, value: &'s str, located: bool) {
        self.tokenizer.write(value, located);
        while let Some(token) = self.tokenizer.next_chunk_token() {
            self.tokenizer.errors.clear();
            self.dispatch(token);
            self.errors.clear();
        }
        self.tokenizer.errors.clear();
    }

    pub(crate) fn feed_text(&mut self, value: &str) {
        // A tag left hanging in a raw node is dropped, but text after PLAINTEXT stays text
        if !matches!(
            self.tokenizer.state,
            State::Data
                | State::Rcdata
                | State::Rawtext
                | State::ScriptData
                | State::Plaintext
        ) {
            self.tokenizer.state = State::Data;
        }
        self.reset_tokenizer();
        // Always a character token, never whitespace: hast-util-raw's `text` makes one
        self.process(Token::Character(CharacterToken {
            kind: CharacterKind::Character,
            chars: value.to_string(),
            span: Span::default(),
        }));
        self.errors.clear();
    }

    /// Returns the comment the tree builder inserted, for a stitch to replace.
    pub(crate) fn feed_comment(&mut self, value: &str) -> Option<NodeId> {
        self.reset_tokenizer();
        let before = self.dom.nodes.len();
        self.process(Token::Comment(CommentToken {
            data: value.to_string(),
            span: Span::default(),
        }));
        self.errors.clear();
        let last = self.dom.nodes.len().checked_sub(1)?;
        (last >= before && matches!(self.dom.nodes[last].data, NodeData::Comment(_)))
            .then_some(last)
    }

    pub(crate) fn feed_doctype(&mut self) {
        self.reset_tokenizer();
        self.process(Token::Doctype(DoctypeToken {
            name: Some("html".to_string()),
            force_quirks: false,
            public_id: Some(String::new()),
            system_id: Some(String::new()),
            span: Span::default(),
        }));
        self.errors.clear();
    }

    pub(crate) fn feed_start_tag(&mut self, element: &Element<'_>) {
        if self.tokenizer.state == State::Plaintext {
            return;
        }
        let tag_name = element.tag_name.to_lowercase();
        self.reset_tokenizer();
        let current = self.current().unwrap_or(self.document);
        let svg = self.dom.namespace(current) == Namespace::Svg
            || (self.dom.namespace(current) == Namespace::Html && tag_name == "svg");
        self.process(Token::StartTag(TagToken {
            tag_name: tag_name.clone(),
            tag_id: find_tag_id(&tag_name),
            self_closing: false,
            ack_self_closing: false,
            attrs: create_attributes(element, svg),
            span: Span::default(),
        }));
        self.errors.clear();
        self.tokenizer.last_start_tag_name = tag_name;
    }

    pub(crate) fn feed_end_tag(&mut self, element: &Element<'_>) {
        let tag_name = element.tag_name.to_lowercase();
        if (!self.tokenizer.in_foreign_node && VOID_ELEMENTS.contains(&tag_name.as_str()))
            || self.tokenizer.state == State::Plaintext
        {
            return;
        }
        self.reset_tokenizer();
        self.process(Token::EndTag(TagToken {
            tag_name: tag_name.clone(),
            tag_id: find_tag_id(&tag_name),
            self_closing: false,
            ack_self_closing: false,
            attrs: Vec::new(),
            span: Span::default(),
        }));
        self.errors.clear();
        if tag_name == self.tokenizer.last_start_tag_name
            && matches!(
                self.tokenizer.state,
                State::Rcdata | State::Rawtext | State::ScriptData
            )
        {
            self.tokenizer.state = State::Data;
        }
    }

    /// Flush what the tokenizer holds; the nodes keep no positions, as the chunks share no
    /// input.
    pub(crate) fn finish_feed(&mut self) {
        self.reset_tokenizer();
        self.errors.clear();
        for node in &mut self.dom.nodes {
            node.location = None;
        }
    }

    fn reset_tokenizer(&mut self) {
        if let Some(character) = self.tokenizer.reset() {
            self.process(Token::Character(character));
        }
    }
}

// hast-util-to-parse5's attributes: properties written back as attribute values
fn create_attributes(element: &Element<'_>, svg: bool) -> Vec<Attribute> {
    let schema = if svg { &SVG[..] } else { &HTML[..] };
    let mut attributes = Vec::with_capacity(element.properties.len());
    for property in &element.properties {
        let found = find_property(schema, property.name);
        let skip = match &property.value {
            PropertyValue::Null | PropertyValue::Boolean(false) => true,
            PropertyValue::Number(value) if value.is_nan() => true,
            value => found.kinds & BOOLEAN != 0 && !is_truthy(Some(value)),
        };
        if skip {
            continue;
        }
        let value = match &property.value {
            PropertyValue::Boolean(_) | PropertyValue::Null => String::new(),
            PropertyValue::Number(value) => write_js_number(*value),
            PropertyValue::String(value) => (*value).to_string(),
            PropertyValue::List(items) => {
                let mut items: Vec<String> = items
                    .iter()
                    .map(|item| match item {
                        PropertyItem::Number(value) => write_js_number(*value),
                        PropertyItem::String(value) => (*value).to_string(),
                    })
                    .collect();
                let joined = if found.kinds & COMMA_SEPARATED != 0 {
                    if items.last().is_some_and(String::is_empty) {
                        items.push(String::new());
                    }
                    items.join(", ")
                } else {
                    items.join(" ")
                };
                joined.trim().to_string()
            }
        };
        attributes.push(Attribute {
            name: found.attribute.into_owned(),
            namespace: None,
            prefix: None,
            value,
            span: Span::default(),
        });
    }
    attributes
}
