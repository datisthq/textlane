use textlane_core::mdast::{Node, Text};

use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// Data becomes text, merged into the text before it; a line ending inside phrasing joins it.
pub(crate) fn compile_text(compiler: &mut Compiler, name: Name, kind: Kind) {
    match (name, kind) {
        (Name::DATA, Kind::Enter) => enter_data(compiler),
        (Name::DATA, Kind::Exit) => exit_data(compiler),
        (Name::LINE_ENDING, Kind::Exit) => {
            if matches!(compiler.tail_mut(), Node::Heading(_) | Node::Paragraph(_)) {
                compiler.index -= 1;
                enter_data(compiler);
                compiler.index += 1;
                exit_data(compiler);
            }
        }
        _ => {}
    }
}

fn enter_data(compiler: &mut Compiler) {
    let after_text = compiler
        .tail_mut()
        .children()
        .and_then(|children| children.last())
        .is_some_and(|last| matches!(last, Node::Text(_)));
    if after_text {
        compiler.tail_push_again();
    } else {
        compiler.tail_push(Node::Text(Text {
            value: "",
            position: None,
        }));
    }
}

fn exit_data(compiler: &mut Compiler) {
    let addition = compiler.slice();
    let value = match compiler.tail_mut() {
        Node::Text(text) => text.value,
        _ => return,
    };
    let joined = compiler.join(value, addition);
    if let Node::Text(text) = compiler.tail_mut() {
        text.value = joined;
    }
    compiler.tail_pop();
}
