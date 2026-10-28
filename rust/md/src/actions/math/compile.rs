use textlane_core::mdast::{Math, Node};

use crate::actions::code::compile::trim_line_endings;
use crate::helpers::math::create_math_data;
use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// Math (flow): the opening fence gives `meta`, the lines after it the value.
pub(crate) fn compile_math(compiler: &mut Compiler, name: Name, kind: Kind) {
    match (name, kind) {
        (Name::MATH_FLOW, Kind::Enter) => {
            compiler.tail_push_after_prefix(Node::Math(Math {
                meta: None,
                value: "",
                position: None,
                data: None,
            }));
        }
        (Name::MATH_FLOW_FENCE_META, Kind::Enter) => compiler.buffer(),
        (Name::MATH_FLOW_FENCE_META, Kind::Exit) => {
            let value = compiler.resume();
            if let Node::Math(math) = compiler.tail_mut() {
                math.meta = Some(value);
            }
        }
        (Name::MATH_FLOW_FENCE, Kind::Exit) => {
            if !compiler.code_fence_seen {
                compiler.buffer();
                compiler.code_fence_seen = true;
            }
        }
        (Name::MATH_FLOW, Kind::Exit) => {
            let value = trim_line_endings(compiler.resume(), true);
            let data = create_math_data(compiler.alloc, value);
            if let Node::Math(math) = compiler.tail_mut() {
                math.value = value;
                math.data = Some(data);
            }
            compiler.tail_pop();
            compiler.code_fence_seen = false;
        }
        _ => {}
    }
}
