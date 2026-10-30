use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::message::place::place_messages;
use crate::actions::readability::check::check_readability;
use crate::models::options::Readability;

impl Plugin for Readability {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(ReadabilityCheck {
            options: self.clone(),
        }));
        Ok(())
    }
}

struct ReadabilityCheck {
    options: Readability,
}

impl Transform for ReadabilityCheck {
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
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        match tree {
            Tree::Nlcst(node) => {
                let reports = check_readability(&node, &self.options, alloc);
                place_messages(file, reports);
                Ok(Tree::Nlcst(node))
            }
            _ => Err(Error::new("invalid-stage", "`nl.readability` takes nlcst")),
        }
    }
}
