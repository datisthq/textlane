use textlane_core::ArenaVec;
use textlane_core::mdast::{Delete, Node};

use crate::models::event::Kind;
use crate::utils::compiler::stack::Compiler;

/// Open and close a delete, from GFM strikethrough.
pub(crate) fn compile_delete(compiler: &mut Compiler, kind: Kind) {
    match kind {
        Kind::Enter => compiler.tail_push(Node::Delete(Delete {
            children: ArenaVec::new_in(compiler.alloc),
            position: None,
            data: None,
        })),
        Kind::Exit => compiler.tail_pop(),
    }
}
