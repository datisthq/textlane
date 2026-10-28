use textlane_core::Error;
use textlane_core::mdast::{Emphasis, Node, Strong};

use crate::actions::blockquote::stringify::stringify_blockquote;
use crate::actions::code::stringify::stringify_code;
use crate::actions::definition::stringify::stringify_definition;
use crate::actions::emphasis::stringify::{
    create_emphasis_attention, stringify_emphasis,
};
use crate::actions::hard_break::stringify::stringify_hard_break;
use crate::actions::heading::stringify::stringify_heading;
use crate::actions::html::stringify::stringify_html;
use crate::actions::image::stringify::stringify_image;
use crate::actions::image_reference::stringify::stringify_image_reference;
use crate::actions::inline_code::stringify::stringify_inline_code;
use crate::actions::link::stringify::stringify_link;
use crate::actions::link_reference::stringify::stringify_link_reference;
use crate::actions::list::stringify::stringify_list;
use crate::actions::list_item::stringify::stringify_list_item;
use crate::actions::paragraph::stringify::stringify_paragraph;
use crate::actions::root::stringify::stringify_root;
use crate::actions::strong::stringify::{create_strong_attention, stringify_strong};
use crate::actions::text::stringify::stringify_text;
use crate::actions::thematic_break::stringify::stringify_thematic_break;
use crate::helpers::character::{CharacterKind, classify_character};
use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::format::{format_link_as_autolink, read_html_kind};
use crate::utils::serializer::join::join_siblings;
use crate::utils::serializer::phrasing::{Attention, Item, serialize_items};
use crate::utils::serializer::settings::Settings;
use crate::utils::serializer::sink::{Layer, Sink};
use crate::utils::serializer::unsafe_patterns::Pattern;

// Handlers recurse once per level of the tree; past the red zone the stack grows on the heap
const STACK_RED_ZONE: usize = 64 * 1024;
const STACK_GROWTH: usize = 1024 * 1024;

/// The characters around the output of a node.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Info<'s> {
    pub(crate) before: &'s str,
    pub(crate) after: &'s str,
}

/// A node whose children are being serialized; `node` is `None` for the root a lone
/// phrasing node is wrapped in.
#[derive(Clone, Copy)]
pub(crate) struct Parent<'n, 'a> {
    pub(crate) node: Option<&'n Node<'a>>,
    pub(crate) children: &'n [Node<'a>],
}

impl<'n, 'a> Parent<'n, 'a> {
    pub(crate) fn of(node: &'n Node<'a>) -> Parent<'n, 'a> {
        Parent {
            node: Some(node),
            children: node.children().map_or(&[], |children| children),
        }
    }
}

/// An emphasis or strong: its construct and the sequences it may be written with.
pub(crate) struct AttentionKind {
    pub(crate) construct: Construct,
    pub(crate) sequences: Vec<String>,
}

/// The state of one `md::stringify` run, as mdast-util-to-markdown's `state`.
pub(crate) struct Serializer<'c> {
    pub(crate) settings: &'c Settings,
    pub(crate) patterns: &'c [Pattern],
    pub(crate) stack: Vec<Construct>,
    counts: [u32; Construct::COUNT],
    pub(crate) index_stack: Vec<usize>,
    pub(crate) bullet_current: Option<String>,
    pub(crate) bullet_last_used: Option<String>,
    sinks: Vec<Sink>,
    attentions: Vec<Attention>,
}

impl<'c> Serializer<'c> {
    pub(crate) fn new(settings: &'c Settings, patterns: &'c [Pattern]) -> Serializer<'c> {
        Serializer {
            settings,
            patterns,
            stack: Vec::new(),
            counts: [0; Construct::COUNT],
            index_stack: Vec::new(),
            bullet_current: None,
            bullet_last_used: None,
            sinks: vec![Sink::default()],
            attentions: Vec::new(),
        }
    }

    pub(crate) fn enter(&mut self, construct: Construct) {
        self.stack.push(construct);
        self.counts[construct as usize] += 1;
    }

    pub(crate) fn exit(&mut self) {
        if let Some(construct) = self.stack.pop() {
            self.counts[construct as usize] -= 1;
        }
    }

    pub(crate) fn in_construct(&self, construct: Construct) -> bool {
        self.counts[construct as usize] > 0
    }

    /// Leave phrasing for the duration of `run`, as upstream filters it out of its stack
    /// for references and autolinks.
    pub(crate) fn without_phrasing<T>(&mut self, run: impl FnOnce(&mut Self) -> T) -> T {
        let saved = std::mem::take(&mut self.counts[Construct::Phrasing as usize]);
        let result = run(self);
        self.counts[Construct::Phrasing as usize] = saved;
        result
    }

    pub(crate) fn write(&mut self, value: &str) {
        if let Some(sink) = self.sinks.last_mut() {
            sink.write(value);
        }
    }

    pub(crate) fn open_layer(&mut self, layer: Layer) {
        if let Some(sink) = self.sinks.last_mut() {
            sink.open(layer);
        }
    }

    pub(crate) fn close_layer(&mut self) {
        if let Some(sink) = self.sinks.last_mut() {
            sink.close();
        }
    }

    /// Everything written so far, with prefixes applied.
    pub(crate) fn finish(mut self) -> String {
        self.sinks.pop().map(Sink::finish).unwrap_or_default()
    }

    /// Write `node` into the current output.
    pub(crate) fn handle<'n, 'a>(
        &mut self,
        node: &'n Node<'a>,
        parent: Option<Parent<'n, 'a>>,
        info: Info,
    ) -> Result<(), Error> {
        stacker::maybe_grow(STACK_RED_ZONE, STACK_GROWTH, || match node {
            Node::Root(_) => stringify_root(self, Parent::of(node), info),
            Node::Paragraph(_) => stringify_paragraph(self, Parent::of(node), info),
            Node::Heading(heading) => stringify_heading(self, heading, Parent::of(node)),
            Node::ThematicBreak(_) => {
                stringify_thematic_break(self);
                Ok(())
            }
            Node::Text(text) => {
                stringify_text(self, text, parent, info);
                Ok(())
            }
            Node::Emphasis(_) => stringify_emphasis(self, Parent::of(node), info),
            Node::Strong(_) => stringify_strong(self, Parent::of(node), info),
            Node::Break(_) => {
                stringify_hard_break(self, info);
                Ok(())
            }
            Node::Code(code) => {
                stringify_code(self, code);
                Ok(())
            }
            Node::InlineCode(code) => {
                stringify_inline_code(self, code);
                Ok(())
            }
            Node::Blockquote(_) => stringify_blockquote(self, Parent::of(node)),
            Node::List(list) => stringify_list(self, list, Parent::of(node), parent),
            Node::ListItem(item) => {
                stringify_list_item(self, item, Parent::of(node), parent)
            }
            Node::Link(link) => stringify_link(self, link, Parent::of(node), info),
            Node::Image(image) => {
                stringify_image(self, image);
                Ok(())
            }
            Node::LinkReference(reference) => {
                stringify_link_reference(self, reference, Parent::of(node))
            }
            Node::ImageReference(reference) => {
                stringify_image_reference(self, reference);
                Ok(())
            }
            Node::Definition(definition) => {
                stringify_definition(self, definition);
                Ok(())
            }
            Node::Html(html) => {
                stringify_html(self, html);
                Ok(())
            }
        })
    }

    /// Serialize `node` on its own and return it.
    pub(crate) fn render<'n, 'a>(
        &mut self,
        node: &'n Node<'a>,
        parent: Option<Parent<'n, 'a>>,
        info: Info,
    ) -> Result<String, Error> {
        self.sinks.push(Sink::default());
        let result = self.handle(node, parent, info);
        let sink = self.sinks.pop().unwrap_or_default();
        result.map(|()| sink.finish())
    }

    /// Write `parent`'s flow children, joined by blank lines or what the join rules say.
    pub(crate) fn container_flow(&mut self, parent: Parent) -> Result<(), Error> {
        self.index_stack.push(0);
        let children = parent.children;
        for (index, child) in children.iter().enumerate() {
            if let Some(top) = self.index_stack.last_mut() {
                *top = index;
            }
            self.handle(
                child,
                Some(parent),
                Info {
                    before: "\n",
                    after: "\n",
                },
            )?;
            if !matches!(child, Node::List(_)) {
                self.bullet_last_used = None;
            }
            if let Some(next) = children.get(index + 1) {
                let between = join_siblings(self, child, next, parent);
                self.write(between);
            }
        }
        self.index_stack.pop();
        Ok(())
    }

    /// Serialize `parent`'s phrasing children.
    pub(crate) fn container_phrasing(
        &mut self,
        parent: Parent,
        info: Info,
    ) -> Result<String, Error> {
        let items = self.phrasing(parent, info)?;
        Ok(serialize_items(
            &items,
            &self.attentions,
            info.before,
            info.after,
        ))
    }

    pub(crate) fn phrasing<'n, 'a>(
        &mut self,
        parent: Parent<'n, 'a>,
        info: Info,
    ) -> Result<Vec<Item>, Error> {
        let children = parent.children;
        let mut results: Vec<Item> = Vec::new();
        let mut before = info.before.to_string();
        self.index_stack.push(0);
        for (index, child) in children.iter().enumerate() {
            if let Some(top) = self.index_stack.last_mut() {
                *top = index;
            }
            let after = match children.get(index + 1) {
                Some(next) => self.peek(next, parent)?,
                None => info.after.to_string(),
            };
            // HTML at the start of a line could form a block, so the line ending before it
            // becomes a space
            if let (Some(Item::Text(previous)), Node::Html(html)) =
                (results.last_mut(), child)
                && matches!(before.as_str(), "\r" | "\n")
                && read_html_kind(html.value) != Some(7)
            {
                if previous.ends_with("\r\n") {
                    previous.truncate(previous.len() - 2);
                } else {
                    previous.pop();
                }
                previous.push(' ');
                before = " ".to_string();
            }
            let kind = match child {
                Node::Emphasis(_) => Some(create_emphasis_attention(self.settings)),
                Node::Strong(_) => Some(create_strong_attention(self.settings)),
                _ => None,
            };
            if let Some(kind) = kind {
                let id = self.attention_item(Parent::of(child), kind)?;
                before = last_character(kind_first_sequence(&self.attentions, id));
                results.push(Item::Attention(id));
            } else {
                let value = self.render(
                    child,
                    Some(parent),
                    Info {
                        before: &before,
                        after: &after,
                    },
                )?;
                if value.is_empty() {
                    continue;
                }
                before = last_character(&value);
                results.push(Item::Text(value));
            }
        }
        self.index_stack.pop();
        Ok(results)
    }

    /// Store an emphasis or strong and its phrasing; returns its index in the store.
    pub(crate) fn attention_item(
        &mut self,
        node: Parent,
        kind: AttentionKind,
    ) -> Result<usize, Error> {
        let sequence = kind.sequences.first().cloned().unwrap_or_default();
        self.enter(kind.construct);
        let children = stacker::maybe_grow(STACK_RED_ZONE, STACK_GROWTH, || {
            self.phrasing(
                node,
                Info {
                    before: &sequence,
                    after: &sequence,
                },
            )
        });
        self.exit();
        self.attentions.push(Attention {
            sequences: kind.sequences,
            children: children?,
        });
        Ok(self.attentions.len() - 1)
    }

    /// Serialize a lone emphasis or strong.
    pub(crate) fn serialize_attention(
        &mut self,
        node: Parent,
        kind: AttentionKind,
        info: Info,
    ) -> Result<(), Error> {
        self.enter(Construct::Phrasing);
        let item = self.attention_item(node, kind);
        self.exit();
        let items = [Item::Attention(item?)];
        let value = serialize_items(&items, &self.attentions, info.before, info.after);
        self.write(&value);
        Ok(())
    }

    // The first character `next` would be written with, read without serializing what
    // upstream's `peek` functions can tell from the node alone
    fn peek(&mut self, next: &Node, parent: Parent) -> Result<String, Error> {
        let character = match next {
            Node::Emphasis(Emphasis { .. }) => Some(self.settings.emphasis),
            Node::Strong(Strong { .. }) => Some(self.settings.strong),
            Node::Html(_) => Some('<'),
            Node::Image(_) | Node::ImageReference(_) => Some('!'),
            Node::InlineCode(_) => Some('`'),
            Node::Link(link) => Some(if format_link_as_autolink(link, self.settings) {
                '<'
            } else {
                '['
            }),
            Node::LinkReference(_) => Some('['),
            _ => None,
        };
        if let Some(character) = character {
            return Ok(character.to_string());
        }
        let value = self.render(
            next,
            Some(parent),
            Info {
                before: "",
                after: "",
            },
        )?;
        Ok(value.chars().next().map(String::from).unwrap_or_default())
    }
}

fn kind_first_sequence(store: &[Attention], id: usize) -> &str {
    store
        .get(id)
        .and_then(|attention| attention.sequences.first())
        .map_or("", String::as_str)
}

/// The last character of `value`, as a string.
pub(crate) fn last_character(value: &str) -> String {
    value
        .chars()
        .next_back()
        .map(String::from)
        .unwrap_or_default()
}

/// Classify a character as upstream does from a UTF-16 code unit: nothing, or half of a
/// surrogate pair, is neither whitespace nor punctuation.
pub(crate) fn classify_unit(character: Option<char>) -> CharacterKind {
    match character {
        Some(character) if u32::from(character) <= 0xFFFF => {
            classify_character(Some(character))
        }
        _ => CharacterKind::Other,
    }
}

/// A hexadecimal character reference, `&#x20;`.
pub(crate) fn encode_character_reference(character: char) -> String {
    format!("&#x{:X};", u32::from(character))
}

/// The `type` unist gives `node`.
pub(crate) fn read_node_type(node: &Node) -> &'static str {
    match node {
        Node::Root(_) => "root",
        Node::Paragraph(_) => "paragraph",
        Node::Heading(_) => "heading",
        Node::ThematicBreak(_) => "thematicBreak",
        Node::Text(_) => "text",
        Node::Emphasis(_) => "emphasis",
        Node::Strong(_) => "strong",
        Node::Break(_) => "break",
        Node::Code(_) => "code",
        Node::InlineCode(_) => "inlineCode",
        Node::Blockquote(_) => "blockquote",
        Node::List(_) => "list",
        Node::ListItem(_) => "listItem",
        Node::Link(_) => "link",
        Node::Image(_) => "image",
        Node::LinkReference(_) => "linkReference",
        Node::ImageReference(_) => "imageReference",
        Node::Definition(_) => "definition",
        Node::Html(_) => "html",
    }
}
