use textlane_core::{Attach, Compiler, Error, File, Plugin, Tree, TreeKind};

use crate::actions::nlcst::stringify::stringify_nlcst;

/// `prose.stringify`: nlcst into text.
#[derive(Clone, Debug, Default)]
pub struct Stringify;

impl Plugin for Stringify {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.set_compiler(|_| Ok(Box::new(Stringify)));
        Ok(())
    }
}

impl Compiler for Stringify {
    fn input(&self) -> TreeKind {
        TreeKind::Nlcst
    }

    fn extension(&self) -> &'static str {
        "txt"
    }

    fn compile(&self, tree: &Tree<'_>, _file: &mut File) -> Result<String, Error> {
        match tree {
            Tree::Nlcst(node) => Ok(stringify_nlcst(node)),
            _ => Err(Error::new("invalid-stage", "`prose.stringify` takes nlcst")),
        }
    }
}
