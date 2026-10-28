use crate::actions::blockquote::compile::compile_blockquote;
use crate::actions::character_reference::compile::compile_character_reference;
use crate::actions::code::compile::compile_code;
use crate::actions::definition::compile::compile_definition;
use crate::actions::delete::compile::compile_delete;
use crate::actions::emphasis::compile::compile_emphasis;
use crate::actions::footnote_definition::compile::compile_footnote_definition;
use crate::actions::footnote_reference::compile::compile_footnote_reference;
use crate::actions::gfm_autolink_literal::compile::compile_gfm_autolink_literal;
use crate::actions::gfm_task_list_item_check::compile::compile_gfm_task_list_item_check;
use crate::actions::hard_break::compile::compile_hard_break;
use crate::actions::heading::compile::compile_heading;
use crate::actions::html::compile::compile_html;
use crate::actions::inline_code::compile::compile_inline_code;
use crate::actions::link::compile::compile_link;
use crate::actions::list::compile::compile_list;
use crate::actions::list_item::compile::compile_list_item;
use crate::actions::paragraph::compile::compile_paragraph;
use crate::actions::strong::compile::compile_strong;
use crate::actions::table::compile::compile_table;
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
        Name::GFM_FOOTNOTE_DEFINITION | Name::GFM_FOOTNOTE_DEFINITION_LABEL_STRING => {
            compile_footnote_definition(compiler, name, kind)
        }
        Name::GFM_FOOTNOTE_CALL | Name::GFM_FOOTNOTE_CALL_STRING => {
            compile_footnote_reference(compiler, name, kind)
        }
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
        Name::GFM_TABLE | Name::GFM_TABLE_ROW | Name::GFM_TABLE_CELL => {
            compile_table(compiler, name, kind)
        }
        Name::LITERAL_AUTOLINK
        | Name::LITERAL_AUTOLINK_EMAIL
        | Name::LITERAL_AUTOLINK_HTTP
        | Name::LITERAL_AUTOLINK_WWW => {
            compile_gfm_autolink_literal(compiler, name, kind)
        }
        Name::STRIKETHROUGH => compile_delete(compiler, kind),
        Name::TASK_LIST_CHECK_VALUE_CHECKED | Name::TASK_LIST_CHECK_VALUE_UNCHECKED => {
            compile_gfm_task_list_item_check(compiler, name, kind)
        }
        _ => {}
    }
}
