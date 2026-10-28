use crate::models::token::Name;

/// The kind of directive being tokenized, which picks its token names.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DirectiveKind {
    #[default]
    Text,
    Leaf,
    Container,
}

/// State the directive constructs share across their state functions.
#[derive(Debug, Default)]
pub(crate) struct DirectiveState {
    pub(crate) kind: DirectiveKind,
    /// How many colons open a container.
    pub(crate) size: usize,
    /// How many colons a closing fence has so far.
    pub(crate) size_close: usize,
    /// The indent before a container's opening fence, removed from its content lines.
    pub(crate) prefix: usize,
    /// How many brackets are open in a label.
    pub(crate) balance: usize,
    /// How many escaped brackets or backslashes a label has.
    pub(crate) label_size: usize,
    /// The enter of a label's last text chunk.
    pub(crate) label_chunk: Option<usize>,
    /// The quote of a quoted attribute value.
    pub(crate) marker: u8,
    /// Whether the shortcut attribute being tokenized is a class, not an id.
    pub(crate) class: bool,
}

/// The token names of one kind of directive, as micromark-extension-directive names them.
pub(crate) struct DirectiveNames {
    pub(crate) name: Name,
    pub(crate) label: Name,
    pub(crate) label_marker: Name,
    pub(crate) label_string: Name,
    pub(crate) attributes: Name,
    pub(crate) attributes_marker: Name,
    pub(crate) attribute: Name,
    pub(crate) attribute_id: Name,
    pub(crate) attribute_id_marker: Name,
    pub(crate) attribute_id_value: Name,
    pub(crate) attribute_class: Name,
    pub(crate) attribute_class_marker: Name,
    pub(crate) attribute_class_value: Name,
    pub(crate) attribute_name: Name,
    pub(crate) attribute_initializer_marker: Name,
    pub(crate) attribute_value_literal: Name,
    pub(crate) attribute_value: Name,
    pub(crate) attribute_value_marker: Name,
    pub(crate) attribute_value_data: Name,
    /// Whether labels and attributes stay on one line.
    pub(crate) disallow_eol: bool,
}

macro_rules! directive_names {
    ($prefix:literal, $disallow_eol:expr) => {
        DirectiveNames {
            name: Name(concat!($prefix, "Name")),
            label: Name(concat!($prefix, "Label")),
            label_marker: Name(concat!($prefix, "LabelMarker")),
            label_string: Name(concat!($prefix, "LabelString")),
            attributes: Name(concat!($prefix, "Attributes")),
            attributes_marker: Name(concat!($prefix, "AttributesMarker")),
            attribute: Name(concat!($prefix, "Attribute")),
            attribute_id: Name(concat!($prefix, "AttributeId")),
            attribute_id_marker: Name(concat!($prefix, "AttributeIdMarker")),
            attribute_id_value: Name(concat!($prefix, "AttributeIdValue")),
            attribute_class: Name(concat!($prefix, "AttributeClass")),
            attribute_class_marker: Name(concat!($prefix, "AttributeClassMarker")),
            attribute_class_value: Name(concat!($prefix, "AttributeClassValue")),
            attribute_name: Name(concat!($prefix, "AttributeName")),
            attribute_initializer_marker: Name(concat!(
                $prefix,
                "AttributeInitializerMarker"
            )),
            attribute_value_literal: Name(concat!($prefix, "AttributeValueLiteral")),
            attribute_value: Name(concat!($prefix, "AttributeValue")),
            attribute_value_marker: Name(concat!($prefix, "AttributeValueMarker")),
            attribute_value_data: Name(concat!($prefix, "AttributeValueData")),
            disallow_eol: $disallow_eol,
        }
    };
}

pub(crate) const TEXT: DirectiveNames = directive_names!("directiveText", false);
pub(crate) const LEAF: DirectiveNames = directive_names!("directiveLeaf", true);
pub(crate) const CONTAINER: DirectiveNames = directive_names!("directiveContainer", true);

pub(crate) const DIRECTIVE_TEXT: Name = Name("directiveText");
pub(crate) const DIRECTIVE_TEXT_MARKER: Name = Name("directiveTextMarker");
pub(crate) const DIRECTIVE_LEAF: Name = Name("directiveLeaf");
pub(crate) const DIRECTIVE_LEAF_SEQUENCE: Name = Name("directiveLeafSequence");
pub(crate) const DIRECTIVE_CONTAINER: Name = Name("directiveContainer");
pub(crate) const DIRECTIVE_CONTAINER_FENCE: Name = Name("directiveContainerFence");
pub(crate) const DIRECTIVE_CONTAINER_SEQUENCE: Name = Name("directiveContainerSequence");
pub(crate) const DIRECTIVE_CONTAINER_CONTENT: Name = Name("directiveContainerContent");
/// A line of a label's text, tokenized as text that keeps its trailing whitespace.
pub(crate) const CHUNK_TEXT: Name = Name("chunkText");
/// A line of a container's content, tokenized as a document of its own.
pub(crate) const CHUNK_DOCUMENT: Name = Name("chunkDocument");

impl DirectiveKind {
    pub(crate) fn names(self) -> &'static DirectiveNames {
        match self {
            DirectiveKind::Text => &TEXT,
            DirectiveKind::Leaf => &LEAF,
            DirectiveKind::Container => &CONTAINER,
        }
    }
}
