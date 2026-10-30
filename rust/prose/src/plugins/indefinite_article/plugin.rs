use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::article::check::check_articles;
use crate::actions::message::place::place_messages;

/// `prose.indefiniteArticle`: check that `a` and `an` match the sound of the next word.
#[derive(Clone, Debug, Default)]
pub struct IndefiniteArticle;

impl Plugin for IndefiniteArticle {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(IndefiniteArticle));
        Ok(())
    }
}

impl Transform for IndefiniteArticle {
    fn input(&self) -> TreeKind {
        TreeKind::Nlcst
    }

    fn output(&self) -> TreeKind {
        TreeKind::Nlcst
    }

    fn run<'a>(
        &self,
        tree: Tree<'a>,
        file: &mut File,
        _alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        let Tree::Nlcst(node) = tree else {
            return Err(Error::new(
                "invalid-stage",
                "`prose.indefiniteArticle` takes nlcst",
            ));
        };
        let mut reports = Vec::new();
        check_articles(&node, &mut reports)?;
        place_messages(file, reports);
        Ok(Tree::Nlcst(node))
    }
}
