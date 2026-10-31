use textlane_core::{
    Allocator, Attach, Error, File, Parser, Plugin, ScriptParserSlot, Tree, TreeKind,
};

use crate::actions::markdown::parse::ParseSettings;
use crate::models::mdx::MdxScript;
use crate::models::options::{
    CjkFriendlySlot, DefinitionListSlot, DirectiveSlot, FrontmatterSlot, Gfm, GfmSlot,
    Math, MathSlot, Matter, MdxSlot, Parse,
};
use crate::utils::onepass::build::parse_onepass_source;

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
            let cjk = slots.get::<CjkFriendlySlot>().is_some_and(|slot| slot.0);
            let mdx = slots
                .get::<MdxSlot>()
                .and_then(|slot| slot.0.as_ref())
                .map(|_| {
                    MdxScript(
                        slots
                            .get::<ScriptParserSlot>()
                            .and_then(|slot| slot.parser.clone()),
                    )
                });
            Ok(Box::new(MarkdownParser {
                gfm,
                math,
                directive,
                frontmatter,
                definition_list,
                mdx,
                cjk,
            }))
        });
        Ok(())
    }
}

struct MarkdownParser {
    gfm: Option<Gfm>,
    math: Option<Math>,
    directive: bool,
    frontmatter: Vec<Matter>,
    definition_list: bool,
    mdx: Option<MdxScript>,
    cjk: bool,
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
        let settings = ParseSettings {
            gfm: self.gfm.as_ref(),
            math: self.math.as_ref(),
            directive: self.directive,
            frontmatter: &self.frontmatter,
            definition_list: self.definition_list,
            mdx: self.mdx.as_ref(),
            cjk: self.cjk,
        };
        parse_onepass_source(input, &settings, alloc).map(Tree::Mdast)
    }
}
