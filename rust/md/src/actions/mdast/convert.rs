use std::collections::HashMap;

use textlane_core::Allocator;
use textlane_core::hast;
use textlane_core::mdast;

use crate::models::options::ToHtml;
use crate::utils::converter::footnote::{append_footer, create_footer};
use crate::utils::converter::state::Converter;

/// Turn mdast into hast, as remark-rehype does; several nodes come back in a root.
pub fn convert_mdast<'a>(
    node: &mdast::Node<'a>,
    alloc: &'a Allocator,
    options: &ToHtml,
) -> hast::Node<'a> {
    convert_mdast_with(node, alloc, options, false)
}

/// `convert_mdast`, lifting MDX out of paragraphs first with `unravel`, as @mdx-js/mdx's
/// remark-mark-and-unravel does.
pub(crate) fn convert_mdast_with<'a>(
    node: &mdast::Node<'a>,
    alloc: &'a Allocator,
    options: &ToHtml,
    unravel: bool,
) -> hast::Node<'a> {
    let (definitions, footnotes) = find_definitions(node);
    let mut converter = Converter {
        alloc,
        options,
        unravel,
        loose_lists: vec![],
        definitions,
        footnote_order: vec![],
        footnote_counts: HashMap::new(),
    };
    let mut nodes = converter.convert(node);
    // Handlers that return several nodes, or text in place of a reference, return a list,
    // which mdast-util-to-hast wraps in a root
    let list = match node {
        mdast::Node::Break(_) => true,
        mdast::Node::LinkReference(reference) => {
            converter.find_definition(reference.identifier).is_none()
        }
        mdast::Node::ImageReference(reference) => {
            converter.find_definition(reference.identifier).is_none()
        }
        _ => false,
    };
    let mut tree = match nodes.pop() {
        Some(node) if nodes.is_empty() && !list => node,
        last => {
            nodes.extend(last);
            hast::Node::Root(hast::Root {
                children: nodes,
                position: None,
                quirks_mode: None,
                data: None,
            })
        }
    };
    if let Some(footer) = create_footer(&mut converter, &footnotes) {
        append_footer(&mut tree, footer);
    }
    tree
}

type Definitions<'a> = HashMap<String, (&'a str, Option<&'a str>)>;
type FootnoteDefinitions<'n, 'a> = HashMap<String, &'n mdast::FootnoteDefinition<'a>>;

// Every definition and footnote definition in the tree by its identifier in upper case,
// in one walk without recursing; the first of an identifier wins
fn find_definitions<'n, 'a>(
    node: &'n mdast::Node<'a>,
) -> (Definitions<'a>, FootnoteDefinitions<'n, 'a>) {
    let mut definitions = HashMap::new();
    let mut footnotes = HashMap::new();
    let mut stack = vec![node];
    while let Some(node) = stack.pop() {
        match node {
            mdast::Node::Definition(definition) => {
                definitions
                    .entry(definition.identifier.to_uppercase())
                    .or_insert((definition.url, definition.title));
            }
            mdast::Node::FootnoteDefinition(definition) => {
                footnotes
                    .entry(definition.identifier.to_uppercase())
                    .or_insert(definition);
            }
            _ => {}
        }
        if let Some(children) = node.children() {
            stack.extend(children.iter().rev());
        }
    }
    (definitions, footnotes)
}
