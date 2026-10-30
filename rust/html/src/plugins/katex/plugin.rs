use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::hast::typeset::typeset_hast;
use crate::models::options::Katex;

impl Plugin for Katex {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for Katex {
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
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        match tree {
            Tree::Hast(mut node) => {
                typeset_hast(&mut node, self, file, alloc);
                Ok(Tree::Hast(node))
            }
            _ => Err(Error::new("invalid-stage", "`html.katex` takes hast")),
        }
    }
}
