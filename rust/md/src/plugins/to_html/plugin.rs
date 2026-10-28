use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::mdast::convert::convert_mdast;
use crate::models::options::ToHtmlOptions;

/// `md.toHtml`: mdast into hast.
#[derive(Clone, Debug, Default)]
pub struct ToHtml {
    pub options: ToHtmlOptions,
}

/// The `md.toHtml` plugin.
pub fn to_html(options: ToHtmlOptions) -> ToHtml {
    ToHtml { options }
}

impl Plugin for ToHtml {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for ToHtml {
    fn input(&self) -> TreeKind {
        TreeKind::Mdast
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
            Tree::Mdast(node) => {
                Ok(Tree::Hast(convert_mdast(&node, alloc, &self.options)))
            }
            Tree::Hast(_) => Err(Error::new("invalid-stage", "`md.toHtml` takes mdast")),
        }
    }
}
