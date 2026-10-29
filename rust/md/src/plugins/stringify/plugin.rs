use textlane_core::{Attach, Compiler, Error, File, Plugin, Tree, TreeKind};

use crate::actions::frontmatter::stringify::create_frontmatter_patterns;
use crate::actions::markdown::stringify::serialize_tree;
use crate::models::options::{
    DefinitionListSlot, DirectiveSlot, FrontmatterSlot, GfmSlot, MathSlot, MdxSlot,
    StringifyOptions,
};
use crate::utils::serializer::settings::Settings;
use crate::utils::serializer::unsafe_patterns::{
    Pattern, compile_patterns, create_def_list_pattern,
};

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
        // A bad pattern must fail at `use`, not later when the pipeline freezes
        compile_patterns(false, None, false, false, &self.options.unsafe_patterns)?;
        let unsafe_patterns = self.options.unsafe_patterns.clone();
        context.set_compiler(move |slots| {
            let mut settings = settings;
            settings.gfm = slots.get::<GfmSlot>().and_then(|slot| slot.0.clone());
            settings.math = slots.get::<MathSlot>().and_then(|slot| slot.0.clone());
            settings.directive =
                slots.get::<DirectiveSlot>().and_then(|slot| slot.0.clone());
            if let Some(slot) = slots.get::<FrontmatterSlot>() {
                settings.frontmatter = slot.0.clone();
            }
            settings.definition_list =
                slots.get::<DefinitionListSlot>().is_some_and(|slot| slot.0);
            settings.mdx = slots.get::<MdxSlot>().and_then(|slot| slot.0.clone());
            // mdast-util-mdx-jsx sets these, overriding the options: MDX has no indented
            // code and no autolinks
            if settings.mdx.is_some() {
                settings.fences = true;
                settings.resource_link = true;
            }
            // Extensions' patterns come before the options' own, as upstream configures them
            let mut extra = create_frontmatter_patterns(&settings.frontmatter);
            if settings.definition_list {
                extra.push(create_def_list_pattern());
            }
            extra.extend(unsafe_patterns.iter().cloned());
            let patterns = compile_patterns(
                settings.gfm.is_some(),
                settings.math.as_ref(),
                settings.directive.is_some(),
                settings.mdx.is_some(),
                &extra,
            )?;
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
            _ => Err(Error::new("invalid-stage", "`md.stringify` takes mdast")),
        }
    }
}
