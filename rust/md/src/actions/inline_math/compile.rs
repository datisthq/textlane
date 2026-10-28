use textlane_core::mdast::{InlineMath, Node};

use crate::actions::inline_code::compile::strip_padding;
use crate::helpers::math::create_inline_math_data;
use crate::models::event::Kind;
use crate::utils::compiler::stack::Compiler;

/// Math (text); as in a code span, one space or line ending of padding comes off each side.
pub(crate) fn compile_inline_math(compiler: &mut Compiler, kind: Kind) {
    match kind {
        Kind::Enter => {
            compiler.tail_push(Node::InlineMath(InlineMath {
                value: "",
                position: None,
                data: None,
            }));
            compiler.buffer();
        }
        Kind::Exit => {
            let value = strip_padding(compiler.resume());
            let data = create_inline_math_data(compiler.alloc, value);
            if let Node::InlineMath(math) = compiler.tail_mut() {
                math.value = value;
                math.data = Some(data);
            }
            compiler.tail_pop();
        }
    }
}
