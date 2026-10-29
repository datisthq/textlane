use textlane_core::{Allocator, Attach, Error, File, Parser, Plugin, Tree, TreeKind};

use crate::actions::markdown::parse::parse_markdown_with;
use crate::models::options::{
    DefinitionListSlot, DirectiveSlot, FrontmatterSlot, GfmOptions, GfmSlot, MathOptions,
    MathSlot, Matter, ParseOptions,
};

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
            let math = slots.get::<MathSlot>().and_then(|slot| slot.0.clone());
            let directive = slots
                .get::<DirectiveSlot>()
                .is_some_and(|slot| slot.0.is_some());
            let frontmatter = slots
                .get::<FrontmatterSlot>()
                .map(|slot| slot.0.clone())
                .unwrap_or_default();
            let definition_list =
                slots.get::<DefinitionListSlot>().is_some_and(|slot| slot.0);
            Ok(Box::new(MarkdownParser {
                gfm,
                math,
                directive,
                frontmatter,
                definition_list,
            }))
        });
        Ok(())
    }
}

struct MarkdownParser {
    gfm: Option<GfmOptions>,
    math: Option<MathOptions>,
    directive: bool,
    frontmatter: Vec<Matter>,
    definition_list: bool,
}

impl Parser for MarkdownParser {
    fn output(&self) -> TreeKind {
        TreeKind::Mdast
    }

    // micromark's preprocessor drops a byte order mark, so offsets count from after it
    fn source<'a>(&self, input: &'a str) -> &'a str {
        input.strip_prefix('\u{FEFF}').unwrap_or(input)
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
            self.math.as_ref(),
            self.directive,
            &self.frontmatter,
            self.definition_list,
            alloc,
        )))
    }
}
