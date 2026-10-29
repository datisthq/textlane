use std::collections::HashMap;

use textlane_core::hast::Node;
use textlane_core::{Allocator, File, Lines, Message, Point, Position};

use crate::helpers::reason::find_reason;
use crate::models::options::ParseOptions;
use crate::utils::builder::convert::convert_dom;
use crate::utils::builder::parser::Parser;

/// Parse HTML into hast as parse5 and hast-util-from-html do, recording parse errors on
/// `file` when `emit_parse_errors` is set.
pub fn parse_html<'a>(
    input: &str,
    options: &ParseOptions,
    file: &mut File,
    alloc: &'a Allocator,
) -> Node<'a> {
    let mut parser = if options.fragment {
        Parser::for_fragment(input)
    } else {
        Parser::for_document(input)
    };
    parser.run();
    let root = if options.fragment {
        parser.take_fragment()
    } else {
        parser.document
    };
    let lines = (options.emit_parse_errors || options.verbose).then(|| Lines::new(input));
    if options.emit_parse_errors
        && let Some(lines) = &lines
    {
        parser.errors.append(&mut parser.tokenizer.errors);
        for error in std::mem::take(&mut parser.errors) {
            let place = lines.to_unist_position(&Position::span(
                Point::new(error.start, 0),
                Point::new(error.end, 0),
            ));
            file.messages.push(Message {
                reason: find_reason(error.code.as_str()).to_string(),
                source: Some("hast-util-from-html".to_string()),
                rule: Some(error.code.as_str().to_string()),
                place: Some(place),
            });
        }
    }
    let verbose = lines.as_ref().filter(|_| options.verbose);
    convert_dom(
        &parser.dom,
        root,
        input.len(),
        alloc,
        &mut HashMap::new(),
        verbose,
    )
}
