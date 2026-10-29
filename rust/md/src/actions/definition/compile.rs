use textlane_core::mdast::{Definition, Node};

use crate::helpers::identifier::normalize_identifier;
use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// A definition: its label, destination and title.
pub(crate) fn compile_definition(compiler: &mut Compiler, name: Name, kind: Kind) {
    match (name, kind) {
        (Name::DEFINITION, Kind::Enter) => {
            compiler.tail_push_after_prefix(Node::Definition(Definition {
                identifier: "",
                label: None,
                url: "",
                title: None,
                position: None,
                data: None,
            }));
        }
        (Name::DEFINITION, Kind::Exit) => compiler.tail_pop(),
        (_, Kind::Enter) => compiler.buffer(),
        (Name::DEFINITION_LABEL_STRING, Kind::Exit) => {
            let label = compiler.resume();
            let raw = compiler.slice();
            let identifier =
                compiler.alloc_str(&normalize_identifier(raw).to_lowercase());
            if let Node::Definition(definition) = compiler.tail_mut() {
                definition.label = Some(label);
                definition.identifier = identifier;
            }
        }
        (Name::DEFINITION_DESTINATION_STRING, Kind::Exit) => {
            let url = compiler.resume();
            if let Node::Definition(definition) = compiler.tail_mut() {
                definition.url = url;
            }
        }
        (Name::DEFINITION_TITLE_STRING, Kind::Exit) => {
            let title = Some(compiler.resume());
            if let Node::Definition(definition) = compiler.tail_mut() {
                definition.title = title;
            }
        }
        _ => {}
    }
}
