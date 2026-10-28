use textlane_core::{Allocator, Attach, Error, File, Parser, Plugin, Tree, TreeKind};

use crate::actions::markdown::parse::parse_markdown_with;
use crate::models::options::{GfmOptions, GfmSlot, ParseOptions};

/// `md.parse`: Markdown into mdast.
#[derive(Clone, Debug, Default)]
pub struct Parse {
    pub options: ParseOptions,
}

/// The `md.parse` plugin.
pub fn parse(options: ParseOptions) -> Parse {
    Parse { options }
}

impl Plugin for Parse {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.set_parser(|slots| {
            let gfm = slots.get::<GfmSlot>().and_then(|slot| slot.0.clone());
            Ok(Box::new(MarkdownParser { gfm }))
        });
        Ok(())
    }
}

struct MarkdownParser {
    gfm: Option<GfmOptions>,
}

impl Parser for MarkdownParser {
    fn output(&self) -> TreeKind {
        TreeKind::Mdast
    }

    fn parse<'a>(
        &self,
        input: &'a str,
        _file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        Ok(Tree::Mdast(parse_markdown_with(
            input,
            self.gfm.as_ref(),
            alloc,
        )))
    }
}
