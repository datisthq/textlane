use textlane_core::Error;
use textlane_core::hast::Node;

use crate::actions::hast::convert::convert_hast;
use crate::actions::program::build::build_program;
use crate::actions::program::rewrite::rewrite_program;
use crate::actions::program::wrap::wrap_program;
use crate::models::options::CompileOptions;
use crate::utils::printer::state::print_program;

/// Compile hast into JavaScript as @mdx-js/mdx 3.1.1 does after remark-rehype:
/// rehype-recma, recma-document, recma-jsx-rewrite, recma-build-jsx unless `jsx`, then
/// astring. `source` is the document the tree's byte spans point into.
pub fn compile_hast(
    tree: &Node<'_>,
    source: Option<&str>,
    options: &CompileOptions,
) -> Result<String, Error> {
    let (state, program) = convert_hast(tree, source, options)?;
    let mut store = state.store;
    wrap_program(&mut store, program, options, &state.snippets)?;
    rewrite_program(&mut store, program, options);
    if !options.jsx {
        build_program(&mut store, program, options)?;
    }
    Ok(print_program(&store, program))
}
