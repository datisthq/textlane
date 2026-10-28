use textlane_core::mdast::{InlineCode, Node};

use crate::models::event::Kind;
use crate::utils::compiler::stack::Compiler;

/// A code span; one space or line ending of padding comes off each side.
pub(crate) fn compile_inline_code(compiler: &mut Compiler, kind: Kind) {
    match kind {
        Kind::Enter => {
            compiler.tail_push(Node::InlineCode(InlineCode {
                value: "",
                position: None,
            }));
            compiler.buffer();
        }
        Kind::Exit => {
            let value = strip_padding(compiler.resume());
            if let Node::InlineCode(code) = compiler.tail_mut() {
                code.value = value;
            }
            compiler.tail_pop();
        }
    }
}

// As micromark's code text resolver: padding goes when both sides have it and the code is
// not only padding
fn strip_padding(value: &str) -> &str {
    let is_padding = |c: char| matches!(c, ' ' | '\n' | '\r');
    if value.chars().all(is_padding) {
        return value;
    }
    let strip_start = |text: &str| -> Option<usize> {
        match text.as_bytes() {
            [b'\r', b'\n', ..] => Some(2),
            [b' ' | b'\n' | b'\r', ..] => Some(1),
            _ => None,
        }
    };
    let strip_end = |text: &str| -> Option<usize> {
        match text.as_bytes() {
            [.., b'\r', b'\n'] => Some(2),
            [.., b' ' | b'\n' | b'\r'] => Some(1),
            _ => None,
        }
    };
    match (strip_start(value), strip_end(value)) {
        (Some(start), Some(end)) => &value[start..value.len() - end],
        _ => value,
    }
}
