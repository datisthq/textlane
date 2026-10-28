use crate::actions::blockquote::compile::compile_blockquote;
use crate::actions::character_reference::compile::compile_character_reference;
use crate::actions::code::compile::compile_code;
use crate::actions::definition::compile::compile_definition;
use crate::actions::emphasis::compile::compile_emphasis;
use crate::actions::hard_break::compile::compile_hard_break;
use crate::actions::heading::compile::compile_heading;
use crate::actions::html::compile::compile_html;
use crate::actions::inline_code::compile::compile_inline_code;
use crate::actions::link::compile::compile_link;
use crate::actions::list::compile::compile_list;
use crate::actions::list_item::compile::compile_list_item;
use crate::actions::paragraph::compile::compile_paragraph;
use crate::actions::strong::compile::compile_strong;
use crate::actions::text::compile::compile_text;
use crate::actions::thematic_break::compile::compile_thematic_break;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// Hand the current event to the compile action of its token.
pub(crate) fn compile_event(compiler: &mut Compiler) {
    let event = &compiler.events[compiler.index];
    let (name, kind) = (event.name, event.kind);
    match name {
        Name::DATA
        | Name::CHARACTER_ESCAPE
        | Name::CHARACTER_ESCAPE_VALUE
        | Name::LINE_ENDING => {
            compile_text(compiler, name, kind);
        }
        Name::CHARACTER_REFERENCE
        | Name::CHARACTER_REFERENCE_MARKER
        | Name::CHARACTER_REFERENCE_MARKER_NUMERIC
        | Name::CHARACTER_REFERENCE_MARKER_HEXADECIMAL
        | Name::CHARACTER_REFERENCE_VALUE => {
            compile_character_reference(compiler, name, kind)
        }
        Name::CODE_FENCED
        | Name::CODE_FENCED_FENCE
        | Name::CODE_FENCED_FENCE_INFO
        | Name::CODE_FENCED_FENCE_META
        | Name::CODE_INDENTED => compile_code(compiler, name, kind),
        Name::CODE_FLOW_CHUNK | Name::CODE_TEXT_DATA => {
            compile_text(compiler, name, kind)
        }
        Name::CODE_TEXT => compile_inline_code(compiler, kind),
        Name::BLOCK_QUOTE => compile_blockquote(compiler, kind),
        Name::LIST_ORDERED | Name::LIST_UNORDERED | Name::LIST_ITEM_VALUE => {
            compile_list(compiler, name, kind);
        }
        Name::LIST_ITEM => compile_list_item(compiler, kind),
        Name::LINK
        | Name::IMAGE
        | Name::LABEL
        | Name::LABEL_TEXT
        | Name::REFERENCE
        | Name::REFERENCE_STRING
        | Name::RESOURCE
        | Name::RESOURCE_DESTINATION_STRING
        | Name::RESOURCE_TITLE_STRING
        | Name::AUTOLINK
        | Name::AUTOLINK_PROTOCOL
        | Name::AUTOLINK_EMAIL => compile_link(compiler, name, kind),
        Name::DEFINITION
        | Name::DEFINITION_LABEL_STRING
        | Name::DEFINITION_DESTINATION_STRING
        | Name::DEFINITION_TITLE_STRING => compile_definition(compiler, name, kind),
        Name::HTML_FLOW | Name::HTML_TEXT => compile_html(compiler, kind),
        Name::HTML_FLOW_DATA | Name::HTML_TEXT_DATA => compile_text(compiler, name, kind),
        Name::EMPHASIS => compile_emphasis(compiler, kind),
        Name::HARD_BREAK_ESCAPE | Name::HARD_BREAK_TRAILING => {
            compile_hard_break(compiler, kind)
        }
        Name::HEADING_ATX
        | Name::HEADING_ATX_SEQUENCE
        | Name::HEADING_SETEXT
        | Name::HEADING_SETEXT_TEXT
        | Name::HEADING_SETEXT_UNDERLINE_SEQUENCE => {
            compile_heading(compiler, name, kind)
        }
        Name::PARAGRAPH => compile_paragraph(compiler, kind),
        Name::STRONG => compile_strong(compiler, kind),
        Name::THEMATIC_BREAK => compile_thematic_break(compiler, kind),
        _ => {}
    }
}
