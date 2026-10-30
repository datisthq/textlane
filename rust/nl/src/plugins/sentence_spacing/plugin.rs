use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::message::place::place_messages;
use crate::actions::sentence_spacing::check::check_sentence_spacing;
use crate::models::options::{SentenceSpacing, Spacing};

impl Plugin for SentenceSpacing {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(SentenceSpacingCheck {
            expected: read_expected(self),
        }));
        Ok(())
    }
}

struct SentenceSpacingCheck {
    expected: usize,
}

impl Transform for SentenceSpacingCheck {
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
                let reports = check_sentence_spacing(&node, self.expected, alloc);
                place_messages(file, reports);
                Ok(Tree::Nlcst(node))
            }
            _ => Err(Error::new(
                "invalid-stage",
                "`nl.sentenceSpacing` takes nlcst",
            )),
        }
    }
}

// retext-sentence-spacing's `preferred`: anything but two spaces or a line ending is one space
fn read_expected(options: &SentenceSpacing) -> usize {
    match &options.preferred {
        Some(Spacing::Count(count)) if *count == 2.0 => 2,
        Some(Spacing::Name(name)) if name == "double-space" => 2,
        Some(Spacing::Count(count)) if *count == 0.0 => 0,
        Some(Spacing::Name(name)) if name == "newline" => 0,
        _ => 1,
    }
}
