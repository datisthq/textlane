use crate::actions::heading::compile::compile_heading;
use crate::actions::paragraph::compile::compile_paragraph;
use crate::actions::text::compile::compile_text;
use crate::actions::thematic_break::compile::compile_thematic_break;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// Hand the current event to the compile action of its token.
pub(crate) fn compile_event(compiler: &mut Compiler) {
    let event = &compiler.events[compiler.index];
    let (name, kind) = (event.name, event.kind);
    match name {
        Name::DATA | Name::LINE_ENDING => compile_text(compiler, name, kind),
        Name::HEADING_ATX | Name::HEADING_ATX_SEQUENCE => {
            compile_heading(compiler, name, kind)
        }
        Name::PARAGRAPH => compile_paragraph(compiler, kind),
        Name::THEMATIC_BREAK => compile_thematic_break(compiler, kind),
        _ => {}
    }
}
