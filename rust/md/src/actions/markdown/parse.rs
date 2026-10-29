use textlane_core::{Allocator, ArenaVec, Error};

use crate::models::mdx::MdxScript;
use crate::models::options::{GfmOptions, MathOptions, Matter};
use textlane_core::mdast::{Node, Root};

use crate::plugins::gfm::actions::text::autolink::autolink_text;
use crate::utils::compiler::handlers::compile_event;
use crate::utils::compiler::stack::Compiler;
use crate::utils::tokenizer::preprocess::tokenize_input;

/// Parse Markdown into mdast, allocated in `alloc` and borrowing `input` where it can; as
/// micromark, it skips a leading byte order mark, and positions count from after it.
pub fn parse_markdown<'a>(input: &'a str, alloc: &'a Allocator) -> Node<'a> {
    let input = input.strip_prefix('\u{FEFF}').unwrap_or(input);
    let settings = ParseSettings::default();
    // Only MDX's constructs fail, and they are off here
    parse_markdown_with(input, &settings, alloc).unwrap_or_else(|_| {
        Node::Root(Root {
            children: ArenaVec::new_in(alloc),
            position: None,
            data: None,
        })
    })
}

/// The syntax extensions a parse has on.
#[derive(Default)]
pub(crate) struct ParseSettings<'s> {
    pub(crate) gfm: Option<&'s GfmOptions>,
    pub(crate) math: Option<&'s MathOptions>,
    pub(crate) directive: bool,
    pub(crate) frontmatter: &'s [Matter],
    pub(crate) definition_list: bool,
    pub(crate) mdx: Option<&'s MdxScript>,
}

/// Parse Markdown with the extensions `settings` turns on; MDX's syntax errors fail it.
pub(crate) fn parse_markdown_with<'a>(
    input: &'a str,
    settings: &ParseSettings,
    alloc: &'a Allocator,
) -> Result<Node<'a>, Error> {
    let (events, prefixes) = tokenize_input(
        input,
        settings.gfm,
        settings.math,
        settings.directive,
        settings.frontmatter,
        settings.definition_list,
        settings.mdx,
    )?;
    let mut compiler = Compiler::new(&events, input, alloc, prefixes);
    compiler.frontmatter = settings.frontmatter;
    for index in 0..events.len() {
        compiler.index = index;
        compile_event(&mut compiler);
        if let Some(error) = compiler.error.take() {
            return Err(error);
        }
    }
    let mut root = compiler.finish()?;
    if settings.gfm.is_some() {
        autolink_text(&mut root, alloc);
    }
    Ok(root)
}
