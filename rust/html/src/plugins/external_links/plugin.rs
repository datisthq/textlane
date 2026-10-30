use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::hast::external::mark_external_links;
use crate::models::options::ExternalLinks;

impl Plugin for ExternalLinks {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for ExternalLinks {
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
                mark_external_links(&mut node, self, alloc)?;
                Ok(Tree::Hast(node))
            }
            _ => Err(Error::new(
                "invalid-stage",
                "`html.externalLinks` takes hast",
            )),
        }
    }
}
