use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::hast::unwrap::unwrap_images;

/// `html.unwrapImages`: paragraphs that hold only images, and links around them, give way to
/// their content, as rehype-unwrap-images does.
#[derive(Clone, Debug, Default)]
pub struct UnwrapImages;

impl Plugin for UnwrapImages {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(UnwrapImages));
        Ok(())
    }
}

impl Transform for UnwrapImages {
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
                unwrap_images(&mut node, alloc);
                Ok(Tree::Hast(node))
            }
            _ => Err(Error::new(
                "invalid-stage",
                "`html.unwrapImages` takes hast",
            )),
        }
    }
}
