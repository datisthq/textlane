use textlane_core::{Allocator, Attach, Error, File, Parser, Plugin, Tree, TreeKind};

use crate::actions::natural_language::parse::parse_natural_language;
use crate::models::options::Parse;

impl Plugin for Parse {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        let parser = *self;
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
        Ok(Tree::Nlcst(parse_natural_language(input, self, alloc)))
    }
}
