use std::sync::Arc;

use textlane_core::{
    Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind, Typographer,
    TypographerSlot,
};

use crate::models::options::Smartypants;
use crate::plugins::smartypants::actions::text::smarten::smarten_text;

impl Plugin for Smartypants {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        let typographer = context
            .slots_mut()
            .get::<TypographerSlot>()
            .and_then(|slot| slot.typographer.clone())
            .ok_or_else(|| {
                Error::new(
                    "missing-natural-language-parser",
                    "`md.smartypants` needs the natural-language plugins: build textlane with `nl`",
                )
            })?;
        context.add_transform(Box::new(Smartening {
            options: self.clone(),
            typographer,
        }));
        Ok(())
    }
}

struct Smartening {
    options: Smartypants,
    typographer: Arc<dyn Typographer>,
}

impl Transform for Smartening {
    fn input(&self) -> TreeKind {
        TreeKind::Mdast
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
            Tree::Mdast(mut node) => {
                smarten_text(&mut node, &self.options, self.typographer.as_ref(), alloc);
                Ok(Tree::Mdast(node))
            }
            _ => Err(Error::new("invalid-stage", "`md.smartypants` takes mdast")),
        }
    }
}
