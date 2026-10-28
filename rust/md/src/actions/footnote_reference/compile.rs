use textlane_core::mdast::{FootnoteReference, Node};

use crate::helpers::identifier::normalize_identifier;
use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// A call to a GFM footnote, `[^a]`.
pub(crate) fn compile_footnote_reference(
    compiler: &mut Compiler,
    name: Name,
    kind: Kind,
) {
    match (name, kind) {
        (Name::GFM_FOOTNOTE_CALL, Kind::Enter) => {
            compiler.tail_push(Node::FootnoteReference(FootnoteReference {
                identifier: "",
                label: None,
                position: None,
            }));
        }
        (Name::GFM_FOOTNOTE_CALL, Kind::Exit) => compiler.tail_pop(),
        (_, Kind::Enter) => compiler.buffer(),
        (_, Kind::Exit) => {
            let label = compiler.resume();
            let identifier = normalize_identifier(compiler.slice()).to_lowercase();
            let identifier = compiler.alloc_str(&identifier);
            if let Node::FootnoteReference(reference) = compiler.tail_mut() {
                reference.label = Some(label);
                reference.identifier = identifier;
            }
        }
    }
}
