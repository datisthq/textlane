use textlane_core::ArenaVec;
use textlane_core::mdast::{FootnoteDefinition, Node};

use crate::helpers::identifier::normalize_identifier;
use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// A GFM footnote definition: its label, then its flow content.
pub(crate) fn compile_footnote_definition(
    compiler: &mut Compiler,
    name: Name,
    kind: Kind,
) {
    match (name, kind) {
        (Name::GFM_FOOTNOTE_DEFINITION, Kind::Enter) => {
            let children = ArenaVec::new_in(compiler.alloc);
            compiler.tail_push_after_prefix(Node::FootnoteDefinition(
                FootnoteDefinition {
                    identifier: "",
                    label: None,
                    children,
                    position: None,
                    data: None,
                },
            ));
        }
        (Name::GFM_FOOTNOTE_DEFINITION, Kind::Exit) => compiler.tail_pop(),
        (_, Kind::Enter) => compiler.buffer(),
        (_, Kind::Exit) => {
            let label = compiler.resume();
            let identifier = normalize_identifier(compiler.slice()).to_lowercase();
            let identifier = compiler.alloc_str(&identifier);
            if let Node::FootnoteDefinition(definition) = compiler.tail_mut() {
                definition.label = Some(label);
                definition.identifier = identifier;
            }
        }
    }
}
