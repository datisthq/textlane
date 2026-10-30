use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::hast::toc::extract_toc;

/// `html.extractToc`: the headings, nested by rank, as the file's `data.toc`, as
/// @stefanprobst/rehype-extract-toc stores them.
#[derive(Clone, Debug, Default)]
pub struct ExtractToc;

impl Plugin for ExtractToc {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(ExtractToc));
        Ok(())
    }
}

impl Transform for ExtractToc {
    fn input(&self) -> TreeKind {
        TreeKind::Hast
    }

    fn output(&self) -> TreeKind {
        TreeKind::Hast
    }

    fn run<'a>(
        &self,
        tree: Tree<'a>,
        file: &mut File,
        _alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        match tree {
            Tree::Hast(node) => {
                file.data.insert("toc".into(), extract_toc(&node));
                Ok(Tree::Hast(node))
            }
            _ => Err(Error::new("invalid-stage", "`html.extractToc` takes hast")),
        }
    }
}
