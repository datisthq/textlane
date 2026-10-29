use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::hast::sanitize::sanitize_hast;
use crate::models::options::SanitizeOptions;

/// `html.sanitize`: keep only what a schema allows.
#[derive(Clone, Debug, Default)]
pub struct Sanitize {
    pub options: SanitizeOptions,
}

/// The `html.sanitize` plugin.
pub fn sanitize(options: SanitizeOptions) -> Sanitize {
    Sanitize { options }
}

impl Plugin for Sanitize {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for Sanitize {
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
            Tree::Hast(node) => {
                Ok(Tree::Hast(sanitize_hast(&node, &self.options, alloc)))
            }
            _ => Err(Error::new("invalid-stage", "`html.sanitize` takes hast")),
        }
    }
}
