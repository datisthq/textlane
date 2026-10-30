use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::hast::convert::convert_hast;
use crate::models::options::ToMarkdown;

impl Plugin for ToMarkdown {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for ToMarkdown {
    fn input(&self) -> TreeKind {
        TreeKind::Hast
    }

    fn output(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn run<'a>(
        &self,
        tree: Tree<'a>,
        _file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        match tree {
            Tree::Hast(node) => Ok(Tree::Mdast(convert_hast(node, self, alloc))),
            _ => Err(Error::new("invalid-stage", "`html.toMarkdown` takes hast")),
        }
    }
}
