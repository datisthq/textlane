use std::sync::Arc;

use textlane_core::{
    Allocator, Attach, Error, File, NaturalLanguageParser, NaturalLanguageParserSlot,
    Plugin, Transform, Tree, TreeKind,
};

use crate::actions::hast::bridge::bridge_hast;

/// `html.toNaturalLanguage`: hast into nlcst, tokenized by the processor's natural-language parser.
#[derive(Clone, Copy, Debug, Default)]
pub struct ToNaturalLanguage;

struct Bridge {
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
                    "`html.toNaturalLanguage` needs a natural-language parser: build textlane with `nl`",
                )
            })?;
        context.add_transform(Box::new(Bridge { parser }));
        Ok(())
    }
}

impl Transform for Bridge {
    fn input(&self) -> TreeKind {
        TreeKind::Hast
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
            Tree::Hast(node) => Ok(Tree::Nlcst(bridge_hast(
                &node,
                file.value.as_deref().unwrap_or_default(),
                self.parser.as_ref(),
                alloc,
            ))),
            _ => Err(Error::new(
                "invalid-stage",
                "`html.toNaturalLanguage` takes hast",
            )),
        }
    }
}

#[cfg(test)]
#[path = "plugin_unit.rs"]
mod unit;
