use textlane_core::{Attach, Compiler, Error, File, Plugin, Tree, TreeKind};

use crate::actions::hast::compile::compile_hast;
use crate::models::options::CompileOptions;

/// `mdx.compile`: hast into a JavaScript module, as @mdx-js/mdx's `compile` makes it.
#[derive(Clone, Debug, Default)]
pub struct Compile {
    pub options: CompileOptions,
}

/// The `mdx.compile` plugin.
pub fn compile(options: CompileOptions) -> Compile {
    Compile { options }
}

impl Plugin for Compile {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        let compiler = self.clone();
        context.set_compiler(move |_| Ok(Box::new(compiler)));
        Ok(())
    }
}

impl Compiler for Compile {
    fn input(&self) -> TreeKind {
        TreeKind::Hast
    }

    fn extension(&self) -> &'static str {
        "js"
    }

    fn compile(&self, tree: &Tree<'_>, file: &mut File) -> Result<String, Error> {
        match tree {
            Tree::Hast(node) => compile_hast(node, file.value.as_deref(), &self.options),
            _ => Err(Error::new("invalid-stage", "`mdx.compile` takes hast")),
        }
    }
}
