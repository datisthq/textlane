use textlane_core::Allocator;
use textlane_core::mdast::Node;

use crate::models::mdx::MdxScript;
use crate::models::options::{Gfm, Math, Matter};
use crate::utils::onepass::build::parse_onepass;

/// Parse Markdown into mdast, allocated in `alloc` and borrowing `input` where it can; as
/// micromark, it skips a leading byte order mark, and positions count from after it.
pub fn parse_markdown<'a>(input: &'a str, alloc: &'a Allocator) -> Node<'a> {
    parse_onepass(input, alloc)
}

/// The syntax extensions a parse has on.
#[derive(Default)]
pub(crate) struct ParseSettings<'s> {
    pub(crate) gfm: Option<&'s Gfm>,
    pub(crate) math: Option<&'s Math>,
    pub(crate) directive: bool,
    pub(crate) frontmatter: &'s [Matter],
    pub(crate) definition_list: bool,
    pub(crate) mdx: Option<&'s MdxScript>,
}
