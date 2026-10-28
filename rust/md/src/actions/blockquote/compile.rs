use textlane_core::ArenaVec;
use textlane_core::mdast::{Blockquote, Node};

use crate::models::event::Kind;
use crate::utils::compiler::stack::Compiler;

/// Open and close a block quote.
pub(crate) fn compile_blockquote(compiler: &mut Compiler, kind: Kind) {
    match kind {
        Kind::Enter => compiler.tail_push_after_prefix(Node::Blockquote(Blockquote {
            children: ArenaVec::new_in(compiler.alloc),
            position: None,
        })),
        Kind::Exit => compiler.tail_pop(),
    }
}
