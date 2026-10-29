use textlane_core::mdast::{MdxJsxAttribute, MdxJsxAttributeValue, MdxJsxElement, Node};
use textlane_core::{ArenaVec, Data, Error, Position};

use crate::helpers::attribute::decode_terminated;
use crate::models::event::Kind;
use crate::models::mdx::MdxTag;
use crate::models::token::Name;
use crate::utils::compiler::stack::{
    Compiler, is_mdx_jsx_tag, serialize_abbreviated_tag, to_point, token_type,
};

const SOURCE: &str = "mdast-util-mdx-jsx";

/// A JSX tag: it opens an element, closes one, or both, as mdast-util-mdx-jsx compiles it.
pub(crate) fn compile_mdx_jsx_element(compiler: &mut Compiler, name: Name, kind: Kind) {
    let part = name
        .0
        .strip_prefix("mdxJsxFlowTag")
        .or_else(|| name.0.strip_prefix("mdxJsxTextTag"))
        .unwrap_or_default();
    match (part, kind) {
        ("", Kind::Enter) => {
            compiler.mdx.tag = Some(MdxTag {
                start: compiler.index,
                ..MdxTag::default()
            });
            compiler.buffer();
        }
        ("", Kind::Exit) => exit_tag(compiler),
        ("ClosingMarker", Kind::Enter) if compiler.mdx.open.is_empty() => {
            fail(
                compiler,
                "unexpected-closing-slash",
                "Unexpected closing slash `/` in tag, expected an open tag first",
            );
        }
        ("ClosingMarker", Kind::Exit) => with_tag(compiler, |tag| tag.close = true),
        ("SelfClosingMarker", Kind::Enter) if is_closing(compiler) => fail(
            compiler,
            "unexpected-self-closing-slash",
            "Unexpected self-closing slash `/` in closing tag, expected the end of the tag",
        ),
        ("SelfClosingMarker", Kind::Exit) => {
            with_tag(compiler, |tag| tag.self_closing = true)
        }
        ("NamePrimary", Kind::Exit) => {
            let value = compiler.slice().to_string();
            with_tag(compiler, |tag| tag.name = Some(value));
        }
        ("NameMember" | "NameLocal", Kind::Exit) => {
            let separator = if part == "NameMember" { '.' } else { ':' };
            let value = compiler.slice();
            with_tag(compiler, |tag| {
                let name = tag.name.get_or_insert_with(String::new);
                name.push(separator);
                name.push_str(value);
            });
        }
        ("Attribute" | "ExpressionAttribute", Kind::Enter) => {
            if is_closing(compiler) {
                fail(
                    compiler,
                    "unexpected-attribute",
                    "Unexpected attribute in closing tag, expected the end of the tag",
                );
                return;
            }
            let point = to_point(&compiler.events[compiler.index].point);
            let position = Some(Position::span(point, point));
            let attribute = if part == "Attribute" {
                MdxJsxAttribute::Attribute {
                    name: "",
                    value: None,
                    position,
                    data: None,
                }
            } else {
                compiler.mdx.value = Some(String::new());
                MdxJsxAttribute::Expression {
                    value: "",
                    position,
                    data: None,
                }
            };
            with_tag(compiler, |tag| tag.attributes.push(attribute));
        }
        ("ExpressionAttribute", Kind::Exit) => {
            let value = take_value(compiler);
            let data = empty_data(compiler);
            let end = to_point(&compiler.events[compiler.index].point);
            with_attribute(compiler, |attribute| {
                if let MdxJsxAttribute::Expression {
                    value: slot,
                    data: data_slot,
                    position,
                } = attribute
                {
                    *slot = value;
                    *data_slot = Some(data);
                    set_end(position, end);
                }
            });
        }
        ("AttributeNamePrimary" | "AttributeNameLocal", Kind::Exit) => {
            let local = part == "AttributeNameLocal";
            let slice = compiler.slice();
            let end = to_point(&compiler.events[compiler.index].point);
            let alloc = compiler.alloc;
            with_attribute(compiler, |attribute| {
                if let MdxJsxAttribute::Attribute { name, position, .. } = attribute {
                    *name = if local {
                        alloc.alloc_str(&format!("{name}:{slice}"))
                    } else {
                        slice
                    };
                    set_end(position, end);
                }
            });
        }
        ("AttributeValueLiteral" | "AttributeValueExpression", Kind::Enter) => {
            compiler.mdx.value = Some(String::new());
        }
        ("AttributeValueLiteral" | "AttributeValueExpression", Kind::Exit) => {
            let value = take_value(compiler);
            let value = if part == "AttributeValueLiteral" {
                MdxJsxAttributeValue::Literal(
                    compiler.alloc_str(&decode_terminated(value)),
                )
            } else {
                MdxJsxAttributeValue::Expression {
                    value,
                    data: Some(empty_data(compiler)),
                }
            };
            let end = to_point(&compiler.events[compiler.index].point);
            with_attribute(compiler, |attribute| {
                if let MdxJsxAttribute::Attribute {
                    value: slot,
                    position,
                    ..
                } = attribute
                {
                    *slot = Some(value);
                    set_end(position, end);
                }
            });
        }
        (
            "AttributeValueLiteralValue"
            | "AttributeValueExpressionValue"
            | "ExpressionAttributeValue",
            Kind::Exit,
        ) => {
            let slice = compiler.slice();
            if let Some(value) = compiler.mdx.value.as_mut() {
                value.push_str(slice);
            }
        }
        _ => {}
    }
}

fn exit_tag(compiler: &mut Compiler) {
    let Some(mut tag) = compiler.mdx.tag.take() else {
        return;
    };
    tag.end = compiler.index;
    if tag.close
        && let Some(tail) = compiler.mdx.open.last()
        && tail.name != tag.name
    {
        let message = format!(
            "Unexpected closing tag `{}`, expected corresponding closing tag for `{}` ({})",
            serialize_abbreviated_tag(tag.name.as_deref(), true),
            serialize_abbreviated_tag(tail.name.as_deref(), false),
            compiler.stringify_span(tail.start, tail.end),
        );
        compiler.error = Some(Error::new(format!("{SOURCE}:end-tag-mismatch"), message));
        return;
    }
    compiler.resume_node();
    if !tag.close {
        let start = to_point(&compiler.events[tag.start].point);
        let mut attributes =
            ArenaVec::with_capacity_in(tag.attributes.len(), compiler.alloc);
        attributes.extend(tag.attributes.drain(..));
        let element = MdxJsxElement {
            name: tag.name.as_deref().map(|name| compiler.alloc_str(name)),
            attributes,
            children: ArenaVec::new_in(compiler.alloc),
            position: Some(Position::span(start, start)),
            data: None,
        };
        let flow = compiler.events[compiler.index].name.0 == "mdxJsxFlowTag";
        let index = compiler.index;
        compiler.index = tag.start;
        compiler.tail_push(if flow {
            Node::MdxJsxFlowElement(element)
        } else {
            Node::MdxJsxTextElement(element)
        });
        compiler.index = index;
    }
    if tag.self_closing || tag.close {
        close_element(compiler, &tag);
        if tag.close {
            compiler.mdx.open.pop();
        }
    } else {
        compiler.mdx.open.push(tag);
    }
}

// Close the innermost element, which must be what this tag closes: any other node open
// inside it fails, as mdast-util-mdx-jsx's `onErrorLeftIsTag` does
fn close_element(compiler: &mut Compiler, tag: &MdxTag) {
    if let Some(open) = compiler.tail_token()
        && (!is_mdx_jsx_tag(compiler.events[open].name)
            || compiler.events[open].name != compiler.events[compiler.index].name)
    {
        let exit = compiler.find_exit(open);
        let kind = token_type(compiler.events[open].name);
        let message = format!(
            "Expected the closing tag `{}` either after the end of `{kind}` ({}) or another opening tag after the start of `{kind}` ({})",
            serialize_abbreviated_tag(tag.name.as_deref(), tag.close),
            compiler.stringify_point(exit),
            compiler.stringify_point(open),
        );
        compiler.error = Some(Error::new(format!("{SOURCE}:end-tag-mismatch"), message));
        return;
    }
    compiler.mdx.closing = true;
    compiler.tail_pop();
    compiler.mdx.closing = false;
}

fn is_closing(compiler: &Compiler) -> bool {
    compiler.mdx.tag.as_ref().is_some_and(|tag| tag.close)
}

fn with_tag<'a>(compiler: &mut Compiler<'a, '_>, change: impl FnOnce(&mut MdxTag<'a>)) {
    if let Some(tag) = compiler.mdx.tag.as_mut() {
        change(tag);
    }
}

fn with_attribute<'a>(
    compiler: &mut Compiler<'a, '_>,
    change: impl FnOnce(&mut MdxJsxAttribute<'a>),
) {
    if let Some(attribute) = compiler
        .mdx
        .tag
        .as_mut()
        .and_then(|tag| tag.attributes.last_mut())
    {
        change(attribute);
    }
}

fn take_value<'a>(compiler: &mut Compiler<'a, '_>) -> &'a str {
    let value = compiler.mdx.value.take().unwrap_or_default();
    compiler.alloc_str(&value)
}

fn empty_data<'a>(compiler: &Compiler<'a, '_>) -> &'a Data<'a> {
    compiler
        .alloc
        .alloc(Data::Object(ArenaVec::new_in(compiler.alloc)))
}

fn set_end<'a>(position: &mut Option<Position<'a>>, end: textlane_core::Point) {
    if let Some(Position::Span { end: slot, .. }) = position {
        *slot = end;
    }
}

fn fail(compiler: &mut Compiler, rule: &str, message: &str) {
    compiler.error = Some(Error::new(format!("{SOURCE}:{rule}"), message));
}
