use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::punctuation::smarten::{Smarten, smarten_punctuation};
use crate::models::smartypants::{Backticks, SmartypantsOptions};

/// `prose.smartypants`: straight quotes, dashes, dots and backticks into typographic ones.
#[derive(Clone, Debug, Default)]
pub struct Smartypants {
    pub options: SmartypantsOptions,
}

/// The `prose.smartypants` plugin.
pub fn smartypants(options: SmartypantsOptions) -> Smartypants {
    Smartypants { options }
}

impl Plugin for Smartypants {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        if self.options.backticks == Backticks::All && self.options.quotes {
            return Err(Error::new(
                "invalid-options",
                "Cannot accept `backticks: 'all'` with `quotes: true`",
            ));
        }
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for Smartypants {
    fn input(&self) -> TreeKind {
        TreeKind::Nlcst
    }

    fn output(&self) -> TreeKind {
        TreeKind::Nlcst
    }

    fn run<'a>(
        &self,
        tree: Tree<'a>,
        _file: &mut File,
        alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        let Tree::Nlcst(mut node) = tree else {
            return Err(Error::new(
                "invalid-stage",
                "`prose.smartypants` takes nlcst",
            ));
        };
        let options = &self.options;
        let quotes = |given: &Option<_>, default: [&'static str; 2]| match given {
            Some(crate::models::smartypants::QuoteCharacters { double, single }) => {
                [&*alloc.alloc_str(double), &*alloc.alloc_str(single)]
            }
            None => default,
        };
        let settings = Smarten {
            quotes: options.quotes,
            ellipses: options.ellipses,
            backticks: options.backticks,
            dashes: options.dashes,
            open: quotes(&options.opening_quotes, ["“", "‘"]),
            close: quotes(&options.closing_quotes, ["”", "’"]),
        };
        smarten_punctuation(&mut node, &settings, alloc);
        Ok(Tree::Nlcst(node))
    }
}
