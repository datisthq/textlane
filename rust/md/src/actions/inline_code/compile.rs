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
                data: None,
            }));
            compiler.buffer();
        }
        Kind::Exit => {
            let mut value = strip_padding(compiler.resume());
            if compiler.in_table && value.contains("\\|") {
                value = compiler.alloc_str(&unescape_pipes(value));
            }
            if let Node::InlineCode(code) = compiler.tail_mut() {
                code.value = value;
            }
            compiler.tail_pop();
        }
    }
}

// As micromark's code text resolver: padding goes when both sides have it and the code is
// not only padding
pub(crate) fn strip_padding(value: &str) -> &str {
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

// As mdast-util-gfm-table: `\|` is `|` in a table, while `\\` keeps both backslashes
fn unescape_pipes(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\\'
            && let Some(next @ ('\\' | '|')) = chars.peek().copied()
        {
            if next == '\\' {
                result.push(character);
            }
            result.push(next);
            chars.next();
            continue;
        }
        result.push(character);
    }
    result
}
