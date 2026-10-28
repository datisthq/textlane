use textlane_core::{Attach, Compiler, Error, File, Plugin, Tree, TreeKind};

use crate::actions::hast::stringify::stringify_hast;
use crate::models::options::StringifyOptions;

/// `html.stringify`: hast into HTML.
#[derive(Clone, Debug, Default)]
pub struct Stringify {
    pub options: StringifyOptions,
}

/// The `html.stringify` plugin.
pub fn stringify(options: StringifyOptions) -> Stringify {
    Stringify { options }
}

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
            Tree::Hast(node) => stringify_hast(node, &self.options),
            Tree::Mdast(_) => {
                Err(Error::new("invalid-stage", "`html.stringify` takes hast"))
            }
        }
    }
}
