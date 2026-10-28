use crate::actions::text::compile::enter_text;
use crate::helpers::reference::decode_reference;
use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// A character reference becomes the text it stands for.
pub(crate) fn compile_character_reference(
    compiler: &mut Compiler,
    name: Name,
    kind: Kind,
) {
    match (name, kind) {
        (Name::CHARACTER_REFERENCE, Kind::Enter) => enter_text(compiler),
        (Name::CHARACTER_REFERENCE, Kind::Exit) => compiler.tail_pop(),
        (Name::CHARACTER_REFERENCE_MARKER, Kind::Exit) => {
            compiler.character_reference_marker = b'&';
        }
        (Name::CHARACTER_REFERENCE_MARKER_NUMERIC, Kind::Exit) => {
            compiler.character_reference_marker = b'#';
        }
        (Name::CHARACTER_REFERENCE_MARKER_HEXADECIMAL, Kind::Exit) => {
            compiler.character_reference_marker = b'x';
        }
        (Name::CHARACTER_REFERENCE_VALUE, Kind::Exit) => {
            let value = compiler.slice();
            let decoded = decode_reference(value, compiler.character_reference_marker)
                .unwrap_or_default();
            let decoded = compiler.alloc.alloc_str(&decoded);
            compiler.append_text(decoded);
            compiler.character_reference_marker = 0;
        }
        _ => {}
    }
}
