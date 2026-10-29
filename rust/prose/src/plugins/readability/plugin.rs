use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::message::place::place_messages;
use crate::actions::readability::check::check_readability;
use crate::models::options::ReadabilityOptions;

/// `prose.readability`: warnings about sentences too hard to read, as retext-readability.
#[derive(Clone, Debug, Default)]
pub struct Readability {
    pub options: ReadabilityOptions,
}

/// The `prose.readability` plugin.
pub fn readability(options: ReadabilityOptions) -> Readability {
    Readability { options }
}

impl Plugin for Readability {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(ReadabilityCheck {
            options: self.options.clone(),
        }));
        Ok(())
    }
}

struct ReadabilityCheck {
    options: ReadabilityOptions,
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
            _ => Err(Error::new(
                "invalid-stage",
                "`prose.readability` takes nlcst",
            )),
        }
    }
}
