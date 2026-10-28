use textlane_core::ArenaVec;
use textlane_core::mdast::{Heading, Node};

use crate::helpers::slice::{Slice, span_of_exit};
use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// Open and close a heading; its opening sequence sets the depth.
pub(crate) fn compile_heading(compiler: &mut Compiler, name: Name, kind: Kind) {
    match (name, kind) {
        (Name::HEADING_ATX, Kind::Enter) => {
            compiler.tail_push_after_prefix(Node::Heading(Heading {
                depth: 0,
                children: ArenaVec::new_in(compiler.alloc),
                position: None,
            }))
        }
        (Name::HEADING_ATX, Kind::Exit) => compiler.tail_pop(),
        (Name::HEADING_ATX_SEQUENCE, Kind::Exit) => {
            let (start, end) = span_of_exit(compiler.events, compiler.index);
            let depth = Slice::from_points(compiler.source.as_bytes(), start, end).len();
            if let Node::Heading(heading) = compiler.tail_mut()
                && heading.depth == 0
            {
                heading.depth = depth.min(6) as u8;
            }
        }
        _ => {}
    }
}
