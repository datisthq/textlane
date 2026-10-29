use textlane_core::mdast::{MdxExpression, MdxjsEsm, Node};
use textlane_core::{ArenaVec, Data};

use crate::models::event::Kind;
use crate::models::mdx::{EXPRESSION_FLOW, MDXJS_ESM};
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// An expression in flow or text, or ESM: a node whose value is the source of its chunks
/// and line endings, as mdast-util-mdx-expression and mdast-util-mdxjs-esm read it.
pub(crate) fn compile_mdx_expression(compiler: &mut Compiler, name: Name, kind: Kind) {
    match kind {
        Kind::Enter
            if name.text().ends_with("Chunk") || name.text().ends_with("Data") => {}
        Kind::Enter => {
            let data = Some(
                &*compiler
                    .alloc
                    .alloc(Data::Object(ArenaVec::new_in(compiler.alloc))),
            );
            let node = if name == MDXJS_ESM {
                Node::MdxjsEsm(MdxjsEsm {
                    value: "",
                    position: None,
                    data,
                })
            } else {
                let expression = MdxExpression {
                    value: "",
                    position: None,
                    data,
                };
                if name == EXPRESSION_FLOW.expression {
                    Node::MdxFlowExpression(expression)
                } else {
                    Node::MdxTextExpression(expression)
                }
            };
            compiler.tail_push(node);
            compiler.mdx.value = Some(String::new());
        }
        Kind::Exit if name.text().ends_with("Chunk") || name.text().ends_with("Data") => {
            let slice = compiler.slice();
            if let Some(value) = compiler.mdx.value.as_mut() {
                value.push_str(slice);
            }
        }
        Kind::Exit => {
            let value = compiler.mdx.value.take().unwrap_or_default();
            let value = compiler.alloc_str(&value);
            match compiler.tail_mut() {
                Node::MdxjsEsm(node) => node.value = value,
                Node::MdxFlowExpression(node) | Node::MdxTextExpression(node) => {
                    node.value = value;
                }
                _ => {}
            }
            compiler.tail_pop();
        }
    }
}
