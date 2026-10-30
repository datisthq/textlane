use textlane_core::Allocator;
use textlane_core::nlcst::Node;

use crate::models::options::{Language, Parse};
use crate::utils::parser::document::{build_paragraph, build_root};
use crate::utils::parser::store::Store;
use crate::utils::parser::tokenize::tokenize_value;

/// Parse natural language into nlcst, as parse-latin's or parse-english's `parse`.
pub fn parse_natural_language<'a>(
    input: &'a str,
    options: &Parse,
    alloc: &'a Allocator,
) -> Node<'a> {
    let mut store = Store::default();
    let content = tokenize_value(&mut store, input, 0);
    let english = options.language == Language::English;
    let paragraph = build_paragraph(&mut store, content, None, english);
    let root = build_root(&mut store, paragraph, None);
    store.export(root, alloc)
}

#[cfg(test)]
#[path = "parse_unit.rs"]
mod unit;
