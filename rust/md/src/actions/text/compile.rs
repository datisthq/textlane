use textlane_core::mdast::{Node, Text};

use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// Data and escaped characters become text, merged into the text before them; a line ending
/// inside phrasing joins it, or the hard break before it.
pub(crate) fn compile_text(compiler: &mut Compiler, name: Name, kind: Kind) {
    match (name, kind) {
        (Name::LINE_ENDING, Kind::Exit) => {
            if compiler.heading_setext_text_after {
                // The line ending between a setext heading's text and its underline
            } else if compiler.hard_break_after {
                compiler.extend_last_child();
                compiler.hard_break_after = false;
            } else if matches!(
                compiler.tail_mut(),
                Node::Delete(_)
                    | Node::Heading(_)
                    | Node::Paragraph(_)
                    | Node::Emphasis(_)
                    | Node::Strong(_)
                    | Node::TextDirective(_)
                    | Node::MdxJsxTextElement(_)
            ) {
                compiler.index -= 1;
                enter_text(compiler);
                compiler.index += 1;
                let value = compiler.slice();
                exit_text(compiler, value);
            }
        }
        (Name::LINE_ENDING, Kind::Enter) => {}
        // An escape's text starts at its backslash but holds only the escaped character
        (Name::CHARACTER_ESCAPE, Kind::Enter) => enter_text(compiler),
        (Name::CHARACTER_ESCAPE, Kind::Exit)
        | (Name::CHARACTER_ESCAPE_VALUE, Kind::Enter) => {}
        (_, Kind::Enter) => enter_text(compiler),
        (_, Kind::Exit) => {
            let value = compiler.slice();
            exit_text(compiler, value);
        }
    }
}

/// Open a text node, or reopen the one right before.
pub(crate) fn enter_text(compiler: &mut Compiler) {
    let after_text = matches!(compiler.last_child_mut(), Some(Node::Text(_)));
    if after_text {
        compiler.tail_push_again();
    } else {
        compiler.tail_push(Node::Text(Text {
            value: "",
            position: None,
            data: None,
        }));
    }
}

/// Append `addition` to the open text node and close it.
pub(crate) fn exit_text<'a>(compiler: &mut Compiler<'a, '_>, addition: &'a str) {
    compiler.append_text(addition);
    compiler.tail_pop();
}
