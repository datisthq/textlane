use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::message::place::place_messages;
use crate::actions::passive::check::check_passive;
use crate::helpers::passive_table::PARTICIPLES;
use crate::models::options::Passive;
use crate::models::phrases::Phrases;

impl Plugin for Passive {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(PassiveCheck {
            participles: index_participles(self),
        }));
        Ok(())
    }
}

struct PassiveCheck {
    participles: (Vec<&'static str>, Phrases),
}

impl Transform for PassiveCheck {
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
                let reports =
                    check_passive(&node, &self.participles.0, &self.participles.1, alloc);
                place_messages(file, reports);
                Ok(Tree::Nlcst(node))
            }
            _ => Err(Error::new("invalid-stage", "`prose.passive` takes nlcst")),
        }
    }
}

// retext-passive's participles without the ignored ones, indexed for the search
fn index_participles(options: &Passive) -> (Vec<&'static str>, Phrases) {
    let ignore = options.ignore.as_deref().unwrap_or_default();
    let participles: Vec<&'static str> = PARTICIPLES
        .iter()
        .copied()
        .filter(|participle| !ignore.iter().any(|ignored| ignored == participle))
        .collect();
    let phrases = Phrases::new(participles.iter().copied(), false);
    (participles, phrases)
}
