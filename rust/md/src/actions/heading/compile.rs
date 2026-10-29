use textlane_core::ArenaVec;
use textlane_core::mdast::{Heading, Node};

use crate::helpers::slice::{Slice, span_of_exit};
use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// Open and close a heading; its opening sequence or its underline sets the depth.
pub(crate) fn compile_heading(compiler: &mut Compiler, name: Name, kind: Kind) {
    match (name, kind) {
        (Name::HEADING_ATX | Name::HEADING_SETEXT, Kind::Enter) => compiler
            .tail_push_after_prefix(Node::Heading(Heading {
                depth: 0,
                children: ArenaVec::new_in(compiler.alloc),
                position: None,
                data: None,
            })),
        (Name::HEADING_ATX, Kind::Exit) => compiler.tail_pop(),
        (Name::HEADING_SETEXT, Kind::Exit) => {
            compiler.heading_setext_text_after = false;
            compiler.tail_pop();
        }
        (Name::HEADING_SETEXT_TEXT, Kind::Exit) => {
            compiler.heading_setext_text_after = true
        }
        // `=` underlines a heading of rank 1, `-` of rank 2
        (Name::HEADING_SETEXT_UNDERLINE_SEQUENCE, Kind::Exit) => {
            let (start, _) = span_of_exit(compiler.events, compiler.index);
            let underline = compiler.source.as_bytes().get(start.index as usize);
            let depth = if underline == Some(&b'-') { 2 } else { 1 };
            if let Node::Heading(heading) = compiler.tail_mut() {
                heading.depth = depth;
            }
        }
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
