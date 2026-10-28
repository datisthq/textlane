use textlane_core::{Attach, Compiler, Error, File, Plugin, Tree, TreeKind};

use crate::actions::markdown::stringify::serialize_tree;
use crate::models::options::StringifyOptions;
use crate::utils::serializer::settings::Settings;
use crate::utils::serializer::unsafe_patterns::{Pattern, compile_patterns};

/// `md.stringify`: mdast into Markdown.
#[derive(Clone, Debug, Default)]
pub struct Stringify {
    pub options: StringifyOptions,
}

/// The `md.stringify` plugin.
pub fn stringify(options: StringifyOptions) -> Stringify {
    Stringify { options }
}

impl Plugin for Stringify {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        let settings = Settings::from_options(&self.options)?;
        let patterns = compile_patterns(&self.options.unsafe_patterns)?;
        context.set_compiler(move |_slots| {
            Ok(Box::new(MarkdownCompiler { settings, patterns }) as Box<dyn Compiler>)
        });
        Ok(())
    }
}

struct MarkdownCompiler {
    settings: Settings,
    patterns: Vec<Pattern>,
}

impl Compiler for MarkdownCompiler {
    fn input(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn extension(&self) -> &'static str {
        "md"
    }

    fn compile(&self, tree: &Tree<'_>, _file: &mut File) -> Result<String, Error> {
        match tree {
            Tree::Mdast(node) => serialize_tree(node, &self.settings, &self.patterns),
            Tree::Hast(_) => {
                Err(Error::new("invalid-stage", "`md.stringify` takes mdast"))
            }
        }
    }
}
