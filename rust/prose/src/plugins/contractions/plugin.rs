use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::contractions::check::check_contractions;
use crate::actions::message::place::place_messages;
use crate::models::contractions::ContractionsOptions;

/// `prose.contractions`: check apostrophes in contractions, such as `don’t`.
#[derive(Clone, Debug, Default)]
pub struct Contractions {
    pub options: ContractionsOptions,
}

/// The `prose.contractions` plugin.
pub fn contractions(options: ContractionsOptions) -> Contractions {
    Contractions { options }
}

impl Plugin for Contractions {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for Contractions {
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
                "`prose.contractions` takes nlcst",
            ));
        };
        let mut reports = Vec::new();
        check_contractions(&node, &self.options, &mut reports);
        place_messages(file, reports);
        Ok(Tree::Nlcst(node))
    }
}
