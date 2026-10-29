use textlane_core::mdast::{Node, ThematicBreak};

use crate::models::event::Kind;
use crate::utils::compiler::stack::Compiler;

/// Open and close a thematic break.
pub(crate) fn compile_thematic_break(compiler: &mut Compiler, kind: Kind) {
    match kind {
        Kind::Enter => {
            compiler.tail_push_after_prefix(Node::ThematicBreak(ThematicBreak {
                position: None,
                data: None,
            }))
        }
        Kind::Exit => compiler.tail_pop(),
    }
}
