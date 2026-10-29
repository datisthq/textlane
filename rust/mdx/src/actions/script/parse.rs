use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use textlane_core::Error;

use crate::actions::script::read::read_script;
use crate::helpers::diagnostic::{check_nesting, with_stack};
use crate::models::estree::{Id, Store};

/// Parse `source` as a module with JSX, as micromark's MDX extensions run acorn: the
/// program and its comments, every offset moved by `base`.
pub(crate) fn parse_script(
    store: &mut Store,
    source: &str,
    base: usize,
) -> Result<(Id, Vec<Id>), Error> {
    let depth =
        check_nesting(source).map_err(|error| Error::new("mdx-failed", error.reason))?;
    // oxc parses and serializes by recursion, as deep as `md.mdx`'s parser allowed
    with_stack(depth, || {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::mjs().with_jsx(true))
            .with_options(ParseOptions {
                preserve_parens: false,
                ..ParseOptions::default()
            })
            .parse();
        if let Some(diagnostic) = parsed.diagnostics.first() {
            return Err(Error::new("mdx-failed", diagnostic.to_string()));
        }
        read_script(store, &parsed.program, source, base)
    })
}
