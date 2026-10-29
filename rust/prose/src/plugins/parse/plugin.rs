use textlane_core::{Allocator, Attach, Error, File, Parser, Plugin, Tree, TreeKind};

use crate::actions::prose::parse::parse_prose;
use crate::models::options::ParseOptions;

/// `prose.parse`: natural language into nlcst.
#[derive(Clone, Debug, Default)]
pub struct Parse {
    pub options: ParseOptions,
}

/// The `prose.parse` plugin.
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
        TreeKind::Nlcst
    }

    fn parse<'a>(
        &self,
        input: &'a str,
        _file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        Ok(Tree::Nlcst(parse_prose(input, &self.options, alloc)))
    }
}
