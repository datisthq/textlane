use textlane_core::mdast::{Break, Node};

use crate::models::event::Kind;
use crate::utils::compiler::stack::Compiler;

/// A hard break, which takes in the line ending after it.
pub(crate) fn compile_hard_break(compiler: &mut Compiler, kind: Kind) {
    match kind {
        Kind::Enter => compiler.tail_push(Node::Break(Break {
            position: None,
            data: None,
        })),
        Kind::Exit => {
            compiler.tail_pop();
            compiler.hard_break_after = true;
        }
    }
}
