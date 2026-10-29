use crate::ArenaVec;
use crate::models::data::Data;
use crate::models::position::Position;

/// A JSX element, `<a b="c">d</a>`, of mdast-util-mdx-jsx; mdast and hast share it, with
/// their own children. A fragment, `<>d</>`, has no `name`.
#[derive(Debug, PartialEq)]
pub struct MdxJsxElement<'a, N> {
    pub name: Option<&'a str>,
    pub attributes: ArenaVec<'a, MdxJsxAttribute<'a>>,
    pub children: ArenaVec<'a, N>,
    pub position: Option<Position<'a>>,
    /// unist's `data`, when the node carries any.
    pub data: Option<&'a Data<'a>>,
}

/// One attribute of a JSX element: `b="c"` (`mdxJsxAttribute`) or `{...d}`
/// (`mdxJsxExpressionAttribute`, its `value` the JavaScript between the braces).
#[derive(Clone, Debug, PartialEq)]
pub enum MdxJsxAttribute<'a> {
    Attribute {
        name: &'a str,
        value: Option<MdxJsxAttributeValue<'a>>,
        position: Option<Position<'a>>,
        data: Option<&'a Data<'a>>,
    },
    Expression {
        value: &'a str,
        position: Option<Position<'a>>,
        data: Option<&'a Data<'a>>,
    },
}

/// The value of a JSX attribute: a string, or `{c}` (`mdxJsxAttributeValueExpression`).
#[derive(Clone, Debug, PartialEq)]
pub enum MdxJsxAttributeValue<'a> {
    Literal(&'a str),
    Expression {
        value: &'a str,
        data: Option<&'a Data<'a>>,
    },
}

/// JavaScript in braces, `{a}`, as source: `mdxFlowExpression` or `mdxTextExpression`.
#[derive(Clone, Debug, PartialEq)]
pub struct MdxExpression<'a> {
    pub value: &'a str,
    pub position: Option<Position<'a>>,
    /// unist's `data`, when the node carries any.
    pub data: Option<&'a Data<'a>>,
}

/// JavaScript imports and exports, `import a from 'b'`, as source.
#[derive(Clone, Debug, PartialEq)]
pub struct MdxjsEsm<'a> {
    pub value: &'a str,
    pub position: Option<Position<'a>>,
    /// unist's `data`, when the node carries any.
    pub data: Option<&'a Data<'a>>,
}
