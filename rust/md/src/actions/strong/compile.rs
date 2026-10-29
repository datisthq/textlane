use textlane_core::ArenaVec;
use textlane_core::mdast::{Node, Strong};

use crate::models::event::Kind;
use crate::utils::compiler::stack::Compiler;

/// Open and close strong emphasis.
pub(crate) fn compile_strong(compiler: &mut Compiler, kind: Kind) {
    match kind {
        Kind::Enter => compiler.tail_push(Node::Strong(Strong {
            children: ArenaVec::new_in(compiler.alloc),
            position: None,
            data: None,
        })),
        Kind::Exit => compiler.tail_pop(),
    }
}
