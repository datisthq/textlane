use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::message::place::place_messages;
use crate::actions::repeated_words::check::check_repeated_words;

/// `prose.repeatedWords`: warnings about a word twice in a row, as retext-repeated-words.
#[derive(Clone, Debug, Default)]
pub struct RepeatedWords;

/// The `prose.repeatedWords` plugin.
pub fn repeated_words() -> RepeatedWords {
    RepeatedWords
}

impl Plugin for RepeatedWords {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(RepeatedWords));
        Ok(())
    }
}

impl Transform for RepeatedWords {
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
                let reports = check_repeated_words(&node, alloc);
                place_messages(file, reports);
                Ok(Tree::Nlcst(node))
            }
            _ => Err(Error::new(
                "invalid-stage",
                "`prose.repeatedWords` takes nlcst",
            )),
        }
    }
}
