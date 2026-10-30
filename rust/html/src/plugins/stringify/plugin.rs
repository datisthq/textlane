use textlane_core::{Attach, Compiler, Error, File, Plugin, Tree, TreeKind};

use crate::actions::hast::stringify::stringify_hast;
use crate::models::options::Stringify;

impl Plugin for Stringify {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        let compiler = self.clone();
        context.set_compiler(move |_| Ok(Box::new(compiler)));
        Ok(())
    }
}

impl Compiler for Stringify {
    fn input(&self) -> TreeKind {
        TreeKind::Hast
    }

    fn extension(&self) -> &'static str {
        "html"
    }

    fn compile(&self, tree: &Tree<'_>, _file: &mut File) -> Result<String, Error> {
        match tree {
            Tree::Hast(node) => stringify_hast(node, self),
            _ => Err(Error::new("invalid-stage", "`html.stringify` takes hast")),
        }
    }
}
