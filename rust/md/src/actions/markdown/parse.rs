use textlane_core::Allocator;

use crate::models::options::{GfmOptions, MathOptions, Matter};
use textlane_core::mdast::Node;

use crate::plugins::gfm::actions::text::autolink::autolink_text;
use crate::utils::compiler::handlers::compile_event;
use crate::utils::compiler::stack::Compiler;
use crate::utils::tokenizer::preprocess::tokenize_input;

/// Parse Markdown into mdast, allocated in `alloc` and borrowing `input` where it can.
pub fn parse_markdown<'a>(input: &'a str, alloc: &'a Allocator) -> Node<'a> {
    parse_markdown_with(input, None, None, false, &[], false, alloc)
}

/// Parse Markdown, with GFM's constructs when `gfm` is given, math's with `math`,
/// directives when `directive` is set, the `frontmatter` matters and definition lists when
/// `definition_list` is set.
pub(crate) fn parse_markdown_with<'a>(
    input: &'a str,
    gfm: Option<&GfmOptions>,
    math: Option<&MathOptions>,
    directive: bool,
    frontmatter: &[Matter],
    definition_list: bool,
    alloc: &'a Allocator,
) -> Node<'a> {
    let (events, prefixes) =
        tokenize_input(input, gfm, math, directive, frontmatter, definition_list);
    let mut compiler = Compiler::new(&events, input, alloc, prefixes);
    compiler.frontmatter = frontmatter;
    for index in 0..events.len() {
        compiler.index = index;
        compile_event(&mut compiler);
    }
    let mut root = compiler.finish();
    if gfm.is_some() {
        autolink_text(&mut root, alloc);
    }
    root
}
