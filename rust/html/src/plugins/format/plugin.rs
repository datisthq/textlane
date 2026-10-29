use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::hast::format::format_hast;
use crate::models::options::FormatOptions;

/// `html.format`: indent and collapse whitespace for readable HTML.
#[derive(Clone, Debug, Default)]
pub struct Format {
    pub options: FormatOptions,
}

/// The `html.format` plugin.
pub fn format(options: FormatOptions) -> Format {
    Format { options }
}

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
            Tree::Hast(node) => Ok(Tree::Hast(format_hast(node, &self.options, alloc))),
            Tree::Mdast(_) => {
                Err(Error::new("invalid-stage", "`html.format` takes hast"))
            }
        }
    }
}
