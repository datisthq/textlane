use std::sync::Arc;

use textlane_core::{
    Allocator, Attach, Error, File, Plugin, ProseParser, ProseParserSlot, Transform,
    Tree, TreeKind,
};

use crate::actions::mdast::bridge::bridge_mdast;
use crate::models::options::ToProse;

struct Bridge {
    options: ToProse,
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
                    "`md.toProse` needs a prose parser: build textlane with `prose`",
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
            _ => Err(Error::new("invalid-stage", "`md.toProse` takes mdast")),
        }
    }
}

#[cfg(test)]
#[path = "plugin_unit.rs"]
mod unit;
