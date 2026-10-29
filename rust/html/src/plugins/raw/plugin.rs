use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::hast::raw::raw_hast;
use crate::models::options::RawOptions;

/// `html.raw`: reparse raw HTML and fix nesting through the HTML parser.
#[derive(Clone, Debug, Default)]
pub struct Raw {
    pub options: RawOptions,
}

/// The `html.raw` plugin.
pub fn raw(options: RawOptions) -> Raw {
    Raw { options }
}

impl Plugin for Raw {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for Raw {
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
            Tree::Hast(node) => Ok(Tree::Hast(raw_hast(&node, &self.options, alloc)?)),
            Tree::Mdast(_) => Err(Error::new("invalid-stage", "`html.raw` takes hast")),
        }
    }
}
