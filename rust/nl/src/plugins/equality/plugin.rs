use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::equality::check::check_equality;
use crate::actions::message::place::place_messages;
use crate::models::equality::EqualityRules;
use crate::models::options::Equality;

impl Plugin for Equality {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(EqualityCheck {
            rules: EqualityRules::new(self),
        }));
        Ok(())
    }
}

struct EqualityCheck {
    rules: EqualityRules,
}

impl Transform for EqualityCheck {
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
                let reports = check_equality(&node, &self.rules, alloc);
                place_messages(file, reports);
                Ok(Tree::Nlcst(node))
            }
            _ => Err(Error::new("invalid-stage", "`nl.equality` takes nlcst")),
        }
    }
}
