use std::fmt;
use std::sync::Arc;

use textlane_core::{Error, ScriptParser};

use crate::models::construct::StateFn;
use crate::models::token::Name;

/// The JavaScript parser `md.mdx` validates expressions and ESM with, when there is one.
#[derive(Clone, Default)]
pub(crate) struct MdxScript(pub(crate) Option<Arc<dyn ScriptParser>>);

impl fmt::Debug for MdxScript {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(if self.0.is_some() {
            "MdxScript(Some)"
        } else {
            "MdxScript(None)"
        })
    }
}

/// State the MDX constructs share across their state functions.
#[derive(Debug)]
pub(crate) struct MdxState {
    /// The token names of the JSX tag being tokenized: in flow or in text.
    pub(crate) jsx: &'static MdxJsxNames,
    /// Where a JSX tag goes once the ECMAScript whitespace it is skipping ends.
    pub(crate) jsx_return: StateFn,
    /// The quote of the attribute value being tokenized.
    pub(crate) jsx_marker: u8,
    /// Whether lazy lines may continue the tag or expression being tokenized.
    pub(crate) allow_lazy: bool,
    /// The token names of the expression being tokenized.
    pub(crate) expression: &'static MdxExpressionNames,
    /// Whether the expression must be one object spread, `{...a}`.
    pub(crate) spread: bool,
    /// Whether the expression may hold nothing but white space and comments.
    pub(crate) allow_empty: bool,
    /// How many braces are open in the expression, counted only without a parser.
    pub(crate) size: usize,
    /// How much line prefix of the expression's current line is eaten.
    pub(crate) prefix: usize,
    /// The line the expression's value starts on.
    pub(crate) line: usize,
    /// The events of the expression or ESM before its value, and how many of them
    /// `source` holds the text of.
    pub(crate) events_start: usize,
    pub(crate) events_read: usize,
    /// The value so far, read once per event, so each try at a closing brace costs only
    /// the parse.
    pub(crate) source: String,
    /// Why the last try to end the expression failed: it is thrown at the end of input.
    pub(crate) crash: Option<Error>,
    /// Whether a try failed before the end of its source: no later brace can fix that.
    pub(crate) hard_crash: bool,
    /// Bytes parsed by the tries at this expression or ESM block so far.
    pub(crate) parsed: usize,
    /// The local names imports declared so far, which later ESM may not declare again.
    pub(crate) imports: Vec<String>,
}

impl MdxState {
    // Whether another try at a value of `len` bytes stays within its re-parse budget: one
    // that only completes after very many tries is given up on, so text keeping it unclosed
    // (`{'` then many `}`) costs linear time, where micromark's is quadratic
    pub(crate) fn spend_parse(&mut self, len: usize) -> bool {
        self.parsed += len;
        self.parsed <= 16 * len + (1 << 20)
    }
}

impl Default for MdxState {
    fn default() -> Self {
        MdxState {
            jsx: &JSX_TEXT,
            jsx_return: |_| crate::models::construct::State::Nok,
            jsx_marker: 0,
            allow_lazy: true,
            expression: &EXPRESSION_TEXT,
            spread: false,
            allow_empty: true,
            size: 0,
            prefix: 0,
            line: 0,
            events_start: 0,
            events_read: 0,
            source: String::new(),
            crash: None,
            hard_crash: false,
            parsed: 0,
            imports: vec![],
        }
    }
}

/// The token names of a JSX tag, as micromark-extension-mdx-jsx names them in flow or text.
#[derive(Debug)]
pub(crate) struct MdxJsxNames {
    pub(crate) tag: Name,
    pub(crate) marker: Name,
    pub(crate) closing_marker: Name,
    pub(crate) self_closing_marker: Name,
    pub(crate) name: Name,
    pub(crate) name_primary: Name,
    pub(crate) name_member_marker: Name,
    pub(crate) name_member: Name,
    pub(crate) name_prefix_marker: Name,
    pub(crate) name_local: Name,
    pub(crate) expression_attribute: MdxExpressionNames,
    pub(crate) attribute: Name,
    pub(crate) attribute_name: Name,
    pub(crate) attribute_name_primary: Name,
    pub(crate) attribute_name_prefix_marker: Name,
    pub(crate) attribute_name_local: Name,
    pub(crate) attribute_initializer_marker: Name,
    pub(crate) attribute_value_literal: Name,
    pub(crate) attribute_value_literal_marker: Name,
    pub(crate) attribute_value_literal_value: Name,
    pub(crate) attribute_value_expression: MdxExpressionNames,
}

/// The token names of an expression: the whole, its braces and the chunks of its value.
#[derive(Debug)]
pub(crate) struct MdxExpressionNames {
    pub(crate) expression: Name,
    pub(crate) marker: Name,
    pub(crate) chunk: Name,
}

macro_rules! jsx_names {
    ($prefix:literal) => {
        MdxJsxNames {
            tag: Name($prefix),
            marker: Name(concat!($prefix, "Marker")),
            closing_marker: Name(concat!($prefix, "ClosingMarker")),
            self_closing_marker: Name(concat!($prefix, "SelfClosingMarker")),
            name: Name(concat!($prefix, "Name")),
            name_primary: Name(concat!($prefix, "NamePrimary")),
            name_member_marker: Name(concat!($prefix, "NameMemberMarker")),
            name_member: Name(concat!($prefix, "NameMember")),
            name_prefix_marker: Name(concat!($prefix, "NamePrefixMarker")),
            name_local: Name(concat!($prefix, "NameLocal")),
            expression_attribute: MdxExpressionNames {
                expression: Name(concat!($prefix, "ExpressionAttribute")),
                marker: Name(concat!($prefix, "ExpressionAttributeMarker")),
                chunk: Name(concat!($prefix, "ExpressionAttributeValue")),
            },
            attribute: Name(concat!($prefix, "Attribute")),
            attribute_name: Name(concat!($prefix, "AttributeName")),
            attribute_name_primary: Name(concat!($prefix, "AttributeNamePrimary")),
            attribute_name_prefix_marker: Name(concat!(
                $prefix,
                "AttributeNamePrefixMarker"
            )),
            attribute_name_local: Name(concat!($prefix, "AttributeNameLocal")),
            attribute_initializer_marker: Name(concat!(
                $prefix,
                "AttributeInitializerMarker"
            )),
            attribute_value_literal: Name(concat!($prefix, "AttributeValueLiteral")),
            attribute_value_literal_marker: Name(concat!(
                $prefix,
                "AttributeValueLiteralMarker"
            )),
            attribute_value_literal_value: Name(concat!(
                $prefix,
                "AttributeValueLiteralValue"
            )),
            attribute_value_expression: MdxExpressionNames {
                expression: Name(concat!($prefix, "AttributeValueExpression")),
                marker: Name(concat!($prefix, "AttributeValueExpressionMarker")),
                chunk: Name(concat!($prefix, "AttributeValueExpressionValue")),
            },
        }
    };
}

pub(crate) static JSX_FLOW: MdxJsxNames = jsx_names!("mdxJsxFlowTag");
pub(crate) static JSX_TEXT: MdxJsxNames = jsx_names!("mdxJsxTextTag");

pub(crate) static EXPRESSION_FLOW: MdxExpressionNames = MdxExpressionNames {
    expression: Name("mdxFlowExpression"),
    marker: Name("mdxFlowExpressionMarker"),
    chunk: Name("mdxFlowExpressionChunk"),
};
pub(crate) static EXPRESSION_TEXT: MdxExpressionNames = MdxExpressionNames {
    expression: Name("mdxTextExpression"),
    marker: Name("mdxTextExpressionMarker"),
    chunk: Name("mdxTextExpressionChunk"),
};

pub(crate) const MDXJS_ESM: Name = Name("mdxjsEsm");
pub(crate) const MDXJS_ESM_DATA: Name = Name("mdxjsEsmData");
pub(crate) const ES_WHITESPACE: Name = Name("esWhitespace");
pub(crate) const LINE_PREFIX: Name = Name("linePrefix");

/// A JSX tag being compiled, as mdast-util-mdx-jsx builds it before it opens or closes an
/// element.
#[derive(Debug, Default)]
pub(crate) struct MdxTag<'a> {
    pub(crate) name: Option<String>,
    pub(crate) attributes: Vec<textlane_core::mdast::MdxJsxAttribute<'a>>,
    pub(crate) close: bool,
    pub(crate) self_closing: bool,
    /// The enter and exit events of the tag.
    pub(crate) start: usize,
    pub(crate) end: usize,
}

/// What compiling MDX keeps between events.
#[derive(Debug, Default)]
pub(crate) struct MdxCompile<'a> {
    /// The tag being compiled.
    pub(crate) tag: Option<MdxTag<'a>>,
    /// The tags whose elements are open, innermost last.
    pub(crate) open: Vec<MdxTag<'a>>,
    /// The value of the expression, attribute value or ESM being read.
    pub(crate) value: Option<String>,
    /// Whether a closing tag is closing its element, which other closes must not do.
    pub(crate) closing: bool,
}
