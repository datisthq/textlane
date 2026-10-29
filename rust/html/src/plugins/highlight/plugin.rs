use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::hast::highlight::highlight_hast;
use crate::models::options::HighlightOptions;

/// `html.highlight`: code blocks colored as Shiki colors them, natively by giallo.
#[derive(Clone, Debug, Default)]
pub struct Highlight {
    pub options: HighlightOptions,
}

/// The `html.highlight` plugin.
pub fn highlight(options: HighlightOptions) -> Highlight {
    Highlight { options }
}

impl Plugin for Highlight {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for Highlight {
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
                highlight_hast(&mut node, &self.options, alloc)?;
                Ok(Tree::Hast(node))
            }
            _ => Err(Error::new("invalid-stage", "`html.highlight` takes hast")),
        }
    }
}
