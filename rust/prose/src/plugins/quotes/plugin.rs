use textlane_core::{Allocator, Attach, Error, File, Plugin, Transform, Tree, TreeKind};

use crate::actions::message::place::place_messages;
use crate::actions::quotes::check::{QuoteMarkers, check_quotes};
use crate::models::quotes::Quotes;

impl Plugin for Quotes {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.add_transform(Box::new(self.clone()));
        Ok(())
    }
}

impl Transform for Quotes {
    fn input(&self) -> TreeKind {
        TreeKind::Nlcst
    }

    fn output(&self) -> TreeKind {
        TreeKind::Nlcst
    }

    fn run<'a>(
        &self,
        tree: Tree<'a>,
        file: &mut File,
        _alloc: &'a Allocator,
    ) -> Result<Tree<'a>, Error> {
        let Tree::Nlcst(node) = tree else {
            return Err(Error::new("invalid-stage", "`prose.quotes` takes nlcst"));
        };
        let read = |given: &Option<Vec<String>>, default: &[&str]| -> Vec<Vec<u16>> {
            match given {
                Some(list) if !list.is_empty() => list
                    .iter()
                    .map(|marker| marker.encode_utf16().collect())
                    .collect(),
                _ => default
                    .iter()
                    .map(|marker| marker.encode_utf16().collect())
                    .collect(),
            }
        };
        let markers = QuoteMarkers {
            preferred: self.preferred.unwrap_or_default(),
            smart: read(&self.smart, &["“”", "‘’"]),
            straight: read(&self.straight, &["\"", "'"]),
        };
        let mut reports = Vec::new();
        check_quotes(&node, &markers, &mut reports);
        place_messages(file, reports);
        Ok(Tree::Nlcst(node))
    }
}
