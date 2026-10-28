use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::hast::convert::convert_hast;
use crate::models::options::ToMarkdownOptions;

/// `html.toMarkdown`: hast into mdast.
#[derive(Clone, Debug, Default)]
pub struct ToMarkdown {
    pub options: ToMarkdownOptions,
}

/// The `html.toMarkdown` plugin.
pub fn to_markdown(options: ToMarkdownOptions) -> ToMarkdown {
    ToMarkdown { options }
}

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
            Tree::Hast(node) => Ok(Tree::Mdast(convert_hast(node, &self.options, alloc))),
            Tree::Mdast(_) => {
                Err(Error::new("invalid-stage", "`html.toMarkdown` takes hast"))
            }
        }
    }
}
