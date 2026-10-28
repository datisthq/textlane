use textlane_core::ArenaVec;
use textlane_core::mdast::{Node, Paragraph};

use crate::models::event::Kind;
use crate::utils::compiler::stack::Compiler;

/// Open and close a paragraph.
pub(crate) fn compile_paragraph(compiler: &mut Compiler, kind: Kind) {
    match kind {
        Kind::Enter => compiler.tail_push_after_prefix(Node::Paragraph(Paragraph {
            children: ArenaVec::new_in(compiler.alloc),
            position: None,
        })),
        Kind::Exit => compiler.tail_pop(),
    }
}
