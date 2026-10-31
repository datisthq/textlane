use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::hast::format::format_hast;
use crate::actions::hast::minify::minify_hast;
use crate::models::options::Format;

impl Plugin for Format {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for Format {
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
            Tree::Hast(mut node) => match self.minify_newlines() {
                Some(newlines) => {
                    minify_hast(&mut node, newlines, alloc);
                    Ok(Tree::Hast(node))
                }
                None => Ok(Tree::Hast(format_hast(node, self, alloc))),
            },
            _ => Err(Error::new("invalid-stage", "`html.format` takes hast")),
        }
    }
}
