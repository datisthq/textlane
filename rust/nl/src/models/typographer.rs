use textlane_core::{Allocator, QuoteCharacters, Typographer, Typography};

use crate::actions::natural_language::parse::parse_natural_language;
use crate::actions::nlcst::stringify::stringify_nlcst;
use crate::actions::punctuation::smarten::{Smarten, smarten_punctuation};
use crate::models::options::Parse;

/// `nl.smartypants` over a text of its own, parsed with parse-latin, as `retext()` parses it.
#[derive(Clone, Copy, Debug, Default)]
pub struct SmartypantsTypographer;

impl Typographer for SmartypantsTypographer {
    fn smarten(&self, text: &str, typography: &Typography) -> String {
        let alloc = Allocator::new();
        let quotes =
            |given: &Option<QuoteCharacters>, default: [&'static str; 2]| match given {
                Some(QuoteCharacters { double, single }) => {
                    [&*alloc.alloc_str(double), &*alloc.alloc_str(single)]
                }
                None => default,
            };
        let settings = Smarten {
            quotes: typography.quotes,
            ellipses: typography.ellipses,
            backticks: typography.backticks,
            dashes: typography.dashes,
            open: quotes(&typography.opening_quotes, ["“", "‘"]),
            close: quotes(&typography.closing_quotes, ["”", "’"]),
        };
        let mut tree = parse_natural_language(text, &Parse::default(), &alloc);
        smarten_punctuation(&mut tree, &settings, &alloc);
        stringify_nlcst(&tree)
    }
}
