use std::sync::Arc;

use textlane_core::{
    Allocator, Attach, Error, File, Plugin, ProseParser, ProseParserSlot, Transform,
    Tree, TreeKind,
};

use crate::actions::hast::bridge::bridge_hast;

/// `html.toProse`: hast into nlcst, tokenized by the processor's prose parser.
#[derive(Clone, Copy, Debug, Default)]
pub struct ToProse;

/// The `html.toProse` plugin.
pub fn to_prose() -> ToProse {
    ToProse
}

struct Bridge {
    parser: Arc<dyn ProseParser>,
}

impl Plugin for ToProse {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        let parser = context
            .slots_mut()
            .get::<ProseParserSlot>()
            .and_then(|slot| slot.parser.clone())
            .ok_or_else(|| {
                Error::new(
                    "missing-prose-parser",
                    "`html.toProse` needs a prose parser: build textlane with `prose`",
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
            _ => Err(Error::new("invalid-stage", "`html.toProse` takes hast")),
        }
    }
}

#[cfg(test)]
#[path = "plugin_unit.rs"]
mod unit;
