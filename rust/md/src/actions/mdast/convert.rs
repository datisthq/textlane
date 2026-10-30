use std::cell::OnceCell;
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
    let mut converter = Converter {
        alloc,
        options,
        unravel,
        loose_lists: vec![],
        root: node,
        definitions: OnceCell::new(),
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
    // The footer lists only called footnotes, so a document that calls none skips the walk
    let footnotes = match converter.footnote_order.is_empty() {
        true => HashMap::new(),
        false => find_footnote_definitions(node),
    };
    if let Some(footer) = create_footer(&mut converter, &footnotes) {
        append_footer(&mut tree, footer);
    }
    tree
}

type Definitions<'a> = HashMap<String, (&'a str, Option<&'a str>)>;
type FootnoteDefinitions<'n, 'a> = HashMap<String, &'n mdast::FootnoteDefinition<'a>>;

// Every definition in the tree by its identifier in upper case, in one walk without
// recursing; the first of an identifier wins
pub(crate) fn find_definitions<'a>(node: &mdast::Node<'a>) -> Definitions<'a> {
    let mut definitions = HashMap::new();
    walk_tree(node, |node| {
        if let mdast::Node::Definition(definition) = node {
            definitions
                .entry(definition.identifier.to_uppercase())
                .or_insert((definition.url, definition.title));
        }
    });
    definitions
}

fn find_footnote_definitions<'n, 'a>(
    node: &'n mdast::Node<'a>,
) -> FootnoteDefinitions<'n, 'a> {
    let mut footnotes = HashMap::new();
    walk_tree(node, |node| {
        if let mdast::Node::FootnoteDefinition(definition) = node {
            footnotes
                .entry(definition.identifier.to_uppercase())
                .or_insert(definition);
        }
    });
    footnotes
}

fn walk_tree<'n, 'a>(
    node: &'n mdast::Node<'a>,
    mut visit: impl FnMut(&'n mdast::Node<'a>),
) {
    let mut stack = vec![node];
    while let Some(node) = stack.pop() {
        visit(node);
        if let Some(children) = node.children() {
            stack.extend(children.iter().rev());
        }
    }
}
