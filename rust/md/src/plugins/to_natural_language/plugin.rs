use std::sync::Arc;

use textlane_core::{
    Allocator, Attach, Error, File, NaturalLanguageParser, NaturalLanguageParserSlot,
    Plugin, Transform, Tree, TreeKind,
};

use crate::actions::mdast::bridge::bridge_mdast;
use crate::models::options::ToNaturalLanguage;

struct Bridge {
    options: ToNaturalLanguage,
    parser: Arc<dyn NaturalLanguageParser>,
}

impl Plugin for ToNaturalLanguage {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        let parser = context
            .slots_mut()
            .get::<NaturalLanguageParserSlot>()
            .and_then(|slot| slot.parser.clone())
            .ok_or_else(|| {
                Error::new(
                    "missing-natural-language-parser",
                    "`md.toNaturalLanguage` needs a natural-language parser: build textlane with `nl`",
                )
            })?;
        context.add_transform(Box::new(Bridge {
            options: self.clone(),
            parser,
        }));
        Ok(())
    }
}

impl Transform for Bridge {
    fn input(&self) -> TreeKind {
        TreeKind::Mdast
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
            Tree::Mdast(node) => Ok(Tree::Nlcst(bridge_mdast(
                &node,
                file.value.as_deref().unwrap_or_default(),
                self.parser.as_ref(),
                &self.options,
                alloc,
            ))),
            _ => Err(Error::new(
                "invalid-stage",
                "`md.toNaturalLanguage` takes mdast",
            )),
        }
    }
}

#[cfg(test)]
#[path = "plugin_unit.rs"]
mod unit;
