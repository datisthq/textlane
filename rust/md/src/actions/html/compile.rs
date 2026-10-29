use textlane_core::mdast::{Html, Node};

use crate::models::event::Kind;
use crate::utils::compiler::stack::Compiler;

/// Raw HTML, block or inline: its data and line endings make the value.
pub(crate) fn compile_html(compiler: &mut Compiler, kind: Kind) {
    match kind {
        Kind::Enter => {
            compiler.tail_push(Node::Html(Html {
                value: "",
                position: None,
                data: None,
            }));
            compiler.buffer();
        }
        Kind::Exit => {
            let value = compiler.resume();
            if let Node::Html(html) = compiler.tail_mut() {
                html.value = value;
            }
            compiler.tail_pop();
        }
    }
}
