use textlane_core::{Allocator, Attach, Error, File, Parser, Plugin, Tree, TreeKind};

use crate::actions::html::parse::parse_html;
use crate::models::options::ParseOptions;

/// `html.parse`: HTML into hast.
#[derive(Clone, Debug, Default)]
pub struct Parse {
    pub options: ParseOptions,
}

/// The `html.parse` plugin.
pub fn parse(options: ParseOptions) -> Parse {
    Parse { options }
}

impl Plugin for Parse {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        let parser = self.clone();
        context.set_parser(move |_| Ok(Box::new(parser)));
        Ok(())
    }
}

impl Parser for Parse {
    fn output(&self) -> TreeKind {
        TreeKind::Hast
    }

    fn parse<'a>(
        &self,
        input: &'a str,
        file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        Ok(Tree::Hast(parse_html(input, &self.options, file, alloc)))
    }
}
