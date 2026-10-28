use crate::ArenaVec;
use crate::models::data::Data;
use crate::models::position::Position;

/// An mdast node.
#[derive(Debug, PartialEq)]
pub enum Node<'a> {
    Root(Root<'a>),
    Paragraph(Paragraph<'a>),
    Heading(Heading<'a>),
    ThematicBreak(ThematicBreak<'a>),
    Text(Text<'a>),
    Emphasis(Emphasis<'a>),
    Strong(Strong<'a>),
    Break(Break<'a>),
    Code(Code<'a>),
    InlineCode(InlineCode<'a>),
    Blockquote(Blockquote<'a>),
    List(List<'a>),
    ListItem(ListItem<'a>),
    Link(Link<'a>),
    Image(Image<'a>),
    LinkReference(LinkReference<'a>),
    ImageReference(ImageReference<'a>),
    Definition(Definition<'a>),
    Html(Html<'a>),
    Delete(Delete<'a>),
    Table(Table<'a>),
    TableRow(TableRow<'a>),
    TableCell(TableCell<'a>),
    FootnoteDefinition(FootnoteDefinition<'a>),
    FootnoteReference(FootnoteReference<'a>),
    Math(Math<'a>),
    InlineMath(InlineMath<'a>),
    ContainerDirective(ContainerDirective<'a>),
    LeafDirective(LeafDirective<'a>),
    TextDirective(TextDirective<'a>),
    Yaml(Yaml<'a>),
    Toml(Toml<'a>),
    Literal(Literal<'a>),
    DefList(DefList<'a>),
    DefListTerm(DefListTerm<'a>),
    DefListDescription(DefListDescription<'a>),
}

/// The document.
#[derive(Debug, PartialEq)]
pub struct Root<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A paragraph.
#[derive(Debug, PartialEq)]
pub struct Paragraph<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A heading of rank `depth`, 1 to 6.
#[derive(Debug, PartialEq)]
pub struct Heading<'a> {
    pub depth: u8,
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A thematic break.
#[derive(Debug, PartialEq)]
pub struct ThematicBreak<'a> {
    pub position: Option<Position<'a>>,
}

/// Stressed text, `*a*`.
#[derive(Debug, PartialEq)]
pub struct Emphasis<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// Strongly stressed text, `**a**`.
#[derive(Debug, PartialEq)]
pub struct Strong<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A hard line break.
#[derive(Debug, PartialEq)]
pub struct Break<'a> {
    pub position: Option<Position<'a>>,
}

/// A block of code, fenced or indented; `lang` and `meta` come from a fence's info string.
#[derive(Debug, PartialEq)]
pub struct Code<'a> {
    pub lang: Option<&'a str>,
    pub meta: Option<&'a str>,
    pub value: &'a str,
    pub position: Option<Position<'a>>,
}

/// A code span.
#[derive(Debug, PartialEq)]
pub struct InlineCode<'a> {
    pub value: &'a str,
    pub position: Option<Position<'a>>,
}

/// A block quote.
#[derive(Debug, PartialEq)]
pub struct Blockquote<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A list: `ordered` from `1.`, starting at `start`; `spread` when its items are separated
/// by blank lines.
#[derive(Debug, PartialEq)]
pub struct List<'a> {
    pub ordered: bool,
    pub start: Option<u32>,
    pub spread: Option<bool>,
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A list item; `checked` is a GFM task's state.
#[derive(Debug, PartialEq)]
pub struct ListItem<'a> {
    pub spread: Option<bool>,
    pub checked: Option<bool>,
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A link to `url`.
#[derive(Debug, PartialEq)]
pub struct Link<'a> {
    pub url: &'a str,
    pub title: Option<&'a str>,
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// An image at `url`.
#[derive(Debug, PartialEq)]
pub struct Image<'a> {
    pub url: &'a str,
    pub title: Option<&'a str>,
    pub alt: Option<&'a str>,
    pub position: Option<Position<'a>>,
}

/// How a reference is written: `[a]`, `[a][]` or `[b][a]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceType {
    Shortcut,
    Collapsed,
    Full,
}

/// A link through a definition, matched by `identifier`.
#[derive(Debug, PartialEq)]
pub struct LinkReference<'a> {
    pub identifier: &'a str,
    pub label: Option<&'a str>,
    pub reference_type: ReferenceType,
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// An image through a definition, matched by `identifier`.
#[derive(Debug, PartialEq)]
pub struct ImageReference<'a> {
    pub identifier: &'a str,
    pub label: Option<&'a str>,
    pub reference_type: ReferenceType,
    pub alt: Option<&'a str>,
    pub position: Option<Position<'a>>,
}

/// A definition of where references with its `identifier` go.
#[derive(Debug, PartialEq)]
pub struct Definition<'a> {
    pub identifier: &'a str,
    pub label: Option<&'a str>,
    pub url: &'a str,
    pub title: Option<&'a str>,
    pub position: Option<Position<'a>>,
}

impl ReferenceType {
    /// The name unist uses.
    pub fn as_str(self) -> &'static str {
        match self {
            ReferenceType::Shortcut => "shortcut",
            ReferenceType::Collapsed => "collapsed",
            ReferenceType::Full => "full",
        }
    }
}

/// Struck-through text, GFM's `~~a~~`.
#[derive(Debug, PartialEq)]
pub struct Delete<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// How a table column is aligned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignKind {
    Left,
    Right,
    Center,
    None,
}

/// A GFM table; `align` holds one alignment per column.
#[derive(Debug, PartialEq)]
pub struct Table<'a> {
    pub align: ArenaVec<'a, AlignKind>,
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A row of a table; the first is its head.
#[derive(Debug, PartialEq)]
pub struct TableRow<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A cell of a table row.
#[derive(Debug, PartialEq)]
pub struct TableCell<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A GFM footnote's content, `[^a]: b`, matched by `identifier`.
#[derive(Debug, PartialEq)]
pub struct FootnoteDefinition<'a> {
    pub identifier: &'a str,
    pub label: Option<&'a str>,
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A call to a GFM footnote, `[^a]`.
#[derive(Debug, PartialEq)]
pub struct FootnoteReference<'a> {
    pub identifier: &'a str,
    pub label: Option<&'a str>,
    pub position: Option<Position<'a>>,
}

/// Math as a block, `$$`; `data` holds the hast `md.toHtml` renders it as.
#[derive(Debug, PartialEq)]
pub struct Math<'a> {
    pub meta: Option<&'a str>,
    pub value: &'a str,
    pub position: Option<Position<'a>>,
    /// unist's `data`, when the node carries any.
    pub data: Option<&'a Data<'a>>,
}

/// Math in text, `$a$`; `data` holds the hast `md.toHtml` renders it as.
#[derive(Debug, PartialEq)]
pub struct InlineMath<'a> {
    pub value: &'a str,
    pub position: Option<Position<'a>>,
    /// unist's `data`, when the node carries any.
    pub data: Option<&'a Data<'a>>,
}

/// A directive's attributes, `{#a .b c=d}`, as name and value pairs in order.
pub type DirectiveAttributes<'a> = ArenaVec<'a, (&'a str, &'a str)>;

/// A container directive, `:::name` … `:::`; `label` is set when the first child is its
/// label, a paragraph marked `data.directiveLabel`.
#[derive(Debug, PartialEq)]
pub struct ContainerDirective<'a> {
    pub name: &'a str,
    pub attributes: DirectiveAttributes<'a>,
    pub label: bool,
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A leaf directive, `::name[label]{attributes}`, its label as its children.
#[derive(Debug, PartialEq)]
pub struct LeafDirective<'a> {
    pub name: &'a str,
    pub attributes: DirectiveAttributes<'a>,
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A text directive, `:name[label]{attributes}`, its label as its children.
#[derive(Debug, PartialEq)]
pub struct TextDirective<'a> {
    pub name: &'a str,
    pub attributes: DirectiveAttributes<'a>,
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A definition list: its terms and their descriptions, in order.
#[derive(Debug, PartialEq)]
pub struct DefList<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A term of a definition list.
#[derive(Debug, PartialEq)]
pub struct DefListTerm<'a> {
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// A description of a definition list; `spread` when a blank line comes before it.
#[derive(Debug, PartialEq)]
pub struct DefListDescription<'a> {
    pub spread: bool,
    pub children: ArenaVec<'a, Node<'a>>,
    pub position: Option<Position<'a>>,
}

/// Raw HTML.
#[derive(Debug, PartialEq)]
pub struct Html<'a> {
    pub value: &'a str,
    pub position: Option<Position<'a>>,
}

/// Plain text.
#[derive(Debug, PartialEq)]
pub struct Text<'a> {
    pub value: &'a str,
    pub position: Option<Position<'a>>,
}

/// YAML frontmatter, without its fences.
#[derive(Debug, PartialEq)]
pub struct Yaml<'a> {
    pub value: &'a str,
    pub position: Option<Position<'a>>,
}

/// TOML frontmatter, without its fences.
#[derive(Debug, PartialEq)]
pub struct Toml<'a> {
    pub value: &'a str,
    pub position: Option<Position<'a>>,
}

/// A node of a type mdast does not define, holding a value: custom frontmatter, say.
#[derive(Debug, PartialEq)]
pub struct Literal<'a> {
    /// The node's `type`.
    pub kind: &'a str,
    pub value: &'a str,
    pub position: Option<Position<'a>>,
}

impl<'a> Node<'a> {
    /// The node's children, when it is a parent.
    pub fn children(&self) -> Option<&ArenaVec<'a, Node<'a>>> {
        match self {
            Node::Root(node) => Some(&node.children),
            Node::Paragraph(node) => Some(&node.children),
            Node::Heading(node) => Some(&node.children),
            Node::Emphasis(node) => Some(&node.children),
            Node::Strong(node) => Some(&node.children),
            Node::Blockquote(node) => Some(&node.children),
            Node::List(node) => Some(&node.children),
            Node::ListItem(node) => Some(&node.children),
            Node::Link(node) => Some(&node.children),
            Node::LinkReference(node) => Some(&node.children),
            Node::Delete(node) => Some(&node.children),
            Node::Table(node) => Some(&node.children),
            Node::TableRow(node) => Some(&node.children),
            Node::TableCell(node) => Some(&node.children),
            Node::FootnoteDefinition(node) => Some(&node.children),
            Node::ContainerDirective(node) => Some(&node.children),
            Node::LeafDirective(node) => Some(&node.children),
            Node::TextDirective(node) => Some(&node.children),
            Node::DefList(node) => Some(&node.children),
            Node::DefListTerm(node) => Some(&node.children),
            Node::DefListDescription(node) => Some(&node.children),
            Node::ThematicBreak(_)
            | Node::FootnoteReference(_)
            | Node::Text(_)
            | Node::Break(_)
            | Node::Code(_)
            | Node::InlineCode(_)
            | Node::Image(_)
            | Node::ImageReference(_)
            | Node::Definition(_)
            | Node::Html(_)
            | Node::Math(_)
            | Node::InlineMath(_)
            | Node::Yaml(_)
            | Node::Toml(_)
            | Node::Literal(_) => None,
        }
    }

    /// The node's children, mutably, when it is a parent.
    pub fn children_mut(&mut self) -> Option<&mut ArenaVec<'a, Node<'a>>> {
        match self {
            Node::Root(node) => Some(&mut node.children),
            Node::Paragraph(node) => Some(&mut node.children),
            Node::Heading(node) => Some(&mut node.children),
            Node::Emphasis(node) => Some(&mut node.children),
            Node::Strong(node) => Some(&mut node.children),
            Node::Blockquote(node) => Some(&mut node.children),
            Node::List(node) => Some(&mut node.children),
            Node::ListItem(node) => Some(&mut node.children),
            Node::Link(node) => Some(&mut node.children),
            Node::LinkReference(node) => Some(&mut node.children),
            Node::Delete(node) => Some(&mut node.children),
            Node::Table(node) => Some(&mut node.children),
            Node::TableRow(node) => Some(&mut node.children),
            Node::TableCell(node) => Some(&mut node.children),
            Node::FootnoteDefinition(node) => Some(&mut node.children),
            Node::ContainerDirective(node) => Some(&mut node.children),
            Node::LeafDirective(node) => Some(&mut node.children),
            Node::TextDirective(node) => Some(&mut node.children),
            Node::DefList(node) => Some(&mut node.children),
            Node::DefListTerm(node) => Some(&mut node.children),
            Node::DefListDescription(node) => Some(&mut node.children),
            Node::ThematicBreak(_)
            | Node::FootnoteReference(_)
            | Node::Text(_)
            | Node::Break(_)
            | Node::Code(_)
            | Node::InlineCode(_)
            | Node::Image(_)
            | Node::ImageReference(_)
            | Node::Definition(_)
            | Node::Html(_)
            | Node::Math(_)
            | Node::InlineMath(_)
            | Node::Yaml(_)
            | Node::Toml(_)
            | Node::Literal(_) => None,
        }
    }

    pub fn position(&self) -> Option<&Position<'a>> {
        match self {
            Node::Root(node) => node.position.as_ref(),
            Node::Paragraph(node) => node.position.as_ref(),
            Node::Heading(node) => node.position.as_ref(),
            Node::ThematicBreak(node) => node.position.as_ref(),
            Node::Text(node) => node.position.as_ref(),
            Node::Emphasis(node) => node.position.as_ref(),
            Node::Strong(node) => node.position.as_ref(),
            Node::Break(node) => node.position.as_ref(),
            Node::Code(node) => node.position.as_ref(),
            Node::InlineCode(node) => node.position.as_ref(),
            Node::Blockquote(node) => node.position.as_ref(),
            Node::List(node) => node.position.as_ref(),
            Node::ListItem(node) => node.position.as_ref(),
            Node::Definition(node) => node.position.as_ref(),
            Node::Html(node) => node.position.as_ref(),
            Node::ImageReference(node) => node.position.as_ref(),
            Node::LinkReference(node) => node.position.as_ref(),
            Node::Image(node) => node.position.as_ref(),
            Node::Link(node) => node.position.as_ref(),
            Node::Delete(node) => node.position.as_ref(),
            Node::Table(node) => node.position.as_ref(),
            Node::TableRow(node) => node.position.as_ref(),
            Node::TableCell(node) => node.position.as_ref(),
            Node::FootnoteDefinition(node) => node.position.as_ref(),
            Node::FootnoteReference(node) => node.position.as_ref(),
            Node::Math(node) => node.position.as_ref(),
            Node::InlineMath(node) => node.position.as_ref(),
            Node::ContainerDirective(node) => node.position.as_ref(),
            Node::LeafDirective(node) => node.position.as_ref(),
            Node::TextDirective(node) => node.position.as_ref(),
            Node::Yaml(node) => node.position.as_ref(),
            Node::Toml(node) => node.position.as_ref(),
            Node::Literal(node) => node.position.as_ref(),
            Node::DefList(node) => node.position.as_ref(),
            Node::DefListTerm(node) => node.position.as_ref(),
            Node::DefListDescription(node) => node.position.as_ref(),
        }
    }

    pub fn position_mut(&mut self) -> &mut Option<Position<'a>> {
        match self {
            Node::Root(node) => &mut node.position,
            Node::Paragraph(node) => &mut node.position,
            Node::Heading(node) => &mut node.position,
            Node::ThematicBreak(node) => &mut node.position,
            Node::Text(node) => &mut node.position,
            Node::Emphasis(node) => &mut node.position,
            Node::Strong(node) => &mut node.position,
            Node::Break(node) => &mut node.position,
            Node::Code(node) => &mut node.position,
            Node::InlineCode(node) => &mut node.position,
            Node::Blockquote(node) => &mut node.position,
            Node::List(node) => &mut node.position,
            Node::ListItem(node) => &mut node.position,
            Node::Definition(node) => &mut node.position,
            Node::Html(node) => &mut node.position,
            Node::ImageReference(node) => &mut node.position,
            Node::LinkReference(node) => &mut node.position,
            Node::Image(node) => &mut node.position,
            Node::Link(node) => &mut node.position,
            Node::Delete(node) => &mut node.position,
            Node::Table(node) => &mut node.position,
            Node::TableRow(node) => &mut node.position,
            Node::TableCell(node) => &mut node.position,
            Node::FootnoteDefinition(node) => &mut node.position,
            Node::FootnoteReference(node) => &mut node.position,
            Node::Math(node) => &mut node.position,
            Node::InlineMath(node) => &mut node.position,
            Node::ContainerDirective(node) => &mut node.position,
            Node::LeafDirective(node) => &mut node.position,
            Node::TextDirective(node) => &mut node.position,
            Node::Yaml(node) => &mut node.position,
            Node::Toml(node) => &mut node.position,
            Node::Literal(node) => &mut node.position,
            Node::DefList(node) => &mut node.position,
            Node::DefListTerm(node) => &mut node.position,
            Node::DefListDescription(node) => &mut node.position,
        }
    }
}
