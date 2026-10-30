use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::hast::slug::slug_hast;
use crate::models::options::Slug;

impl Plugin for Slug {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for Slug {
    fn input(&self) -> TreeKind {
        TreeKind::Hast
    }

    fn output(&self) -> TreeKind {
        TreeKind::Hast
    }

    fn run<'a>(
        &self,
        tree: Tree<'a>,
        _file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        match tree {
            Tree::Hast(mut node) => {
                slug_hast(&mut node, self, alloc);
                Ok(Tree::Hast(node))
            }
            _ => Err(Error::new("invalid-stage", "`html.slug` takes hast")),
        }
    }
}
