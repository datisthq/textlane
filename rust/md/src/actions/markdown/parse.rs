use textlane_core::Allocator;
use textlane_core::mdast::Node;

use crate::utils::compiler::handlers::compile_event;
use crate::utils::compiler::stack::Compiler;
use crate::utils::tokenizer::preprocess::tokenize_input;

/// Parse Markdown into mdast, allocated in `alloc` and borrowing `input` where it can.
pub fn parse_markdown<'a>(input: &'a str, alloc: &'a Allocator) -> Node<'a> {
    let events = tokenize_input(input);
    let mut compiler = Compiler::new(&events, input, alloc);
    for index in 0..events.len() {
        compiler.index = index;
        compile_event(&mut compiler);
    }
    compiler.finish()
}
