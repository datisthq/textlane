use textlane_core::mdast::{Code, Node};

use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// Fenced and indented code: the info string gives `lang` and `meta`, the chunks the value.
pub(crate) fn compile_code(compiler: &mut Compiler, name: Name, kind: Kind) {
    match (name, kind) {
        (Name::CODE_FENCED, Kind::Enter) => compiler.tail_push_after_prefix(empty_code()),
        (Name::CODE_INDENTED, Kind::Enter) => {
            compiler.tail_push(empty_code());
            compiler.buffer();
        }
        (Name::CODE_FENCED_FENCE_INFO | Name::CODE_FENCED_FENCE_META, Kind::Enter) => {
            compiler.buffer();
        }
        (Name::CODE_FENCED_FENCE_INFO, Kind::Exit) => {
            let value = compiler.resume();
            if let Node::Code(code) = compiler.tail_mut() {
                code.lang = Some(value);
            }
        }
        (Name::CODE_FENCED_FENCE_META, Kind::Exit) => {
            let value = compiler.resume();
            if let Node::Code(code) = compiler.tail_mut() {
                code.meta = Some(value);
            }
        }
        // The code itself starts after the opening fence
        (Name::CODE_FENCED_FENCE, Kind::Exit) => {
            if !compiler.code_fence_seen {
                compiler.buffer();
                compiler.code_fence_seen = true;
            }
        }
        (Name::CODE_FENCED | Name::CODE_INDENTED, Kind::Exit) => {
            let fenced = name == Name::CODE_FENCED;
            let value = trim_line_endings(compiler.resume(), fenced);
            if let Node::Code(code) = compiler.tail_mut() {
                code.value = value;
            }
            compiler.tail_pop();
            compiler.code_fence_seen = false;
        }
        _ => {}
    }
}

fn empty_code<'a>() -> Node<'a> {
    Node::Code(Code {
        lang: None,
        meta: None,
        value: "",
        position: None,
        data: None,
    })
}

// The line ending after the opening fence and the one before the end are not code
pub(crate) fn trim_line_endings(value: &str, start: bool) -> &str {
    let value = if start {
        value
            .strip_prefix("\r\n")
            .or_else(|| value.strip_prefix(['\n', '\r']))
            .unwrap_or(value)
    } else {
        value
    };
    value
        .strip_suffix("\r\n")
        .or_else(|| value.strip_suffix(['\n', '\r']))
        .unwrap_or(value)
}
