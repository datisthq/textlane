use textlane_core::mdast::{Node, Text};
use textlane_core::{Allocator, Backticks, Dashes, Ellipses, Typographer, Typography};

use crate::models::options::Smartypants;

enum Piece<'n, 'a> {
    Text(&'n mut Text<'a>),
    Code(usize),
    Block,
}

/// remark-smartypants 3.0.3: quotes over all text at once, joined across phrasing so they
/// pair around links and emphasis, then dashes, dots and backticks per text node.
pub(crate) fn smarten_text<'a>(
    root: &mut Node<'a>,
    options: &Smartypants,
    typographer: &dyn Typographer,
    alloc: &'a Allocator,
) {
    let mut pieces = collect_pieces(root);
    let mut joined = String::new();
    for piece in &pieces {
        match piece {
            Piece::Text(text) => joined.push_str(text.value),
            // Code keeps its length in UTF-16 units, as `"A".repeat(value.length)`
            Piece::Code(length) => joined.extend(std::iter::repeat_n('A', *length)),
            // A block starts fresh, so a quote at its start opens
            Piece::Block => joined.push(' '),
        }
    }
    let quotes = typographer.smarten(&joined, &typography(options, true));
    let units: Vec<u16> = quotes.encode_utf16().collect();
    let rest = typography(options, false);
    let mut start = 0;
    for piece in &mut pieces {
        match piece {
            Piece::Text(text) => {
                let end = start + text.value.encode_utf16().count();
                let slice = &units[start.min(units.len())..end.min(units.len())];
                let value = typographer.smarten(&String::from_utf16_lossy(slice), &rest);
                if value != text.value {
                    text.value = alloc.alloc_str(&value);
                }
                start = end;
            }
            Piece::Code(length) => start += *length,
            Piece::Block => start += 1,
        }
    }
}

// Upstream's first pass changes quotes only, which keeps the joined text's length so it
// splices back; its second everything else, quotes off
fn typography(options: &Smartypants, quotes: bool) -> Typography {
    Typography {
        quotes: quotes && options.quotes,
        ellipses: if quotes {
            Ellipses::Off
        } else {
            options.ellipses
        },
        backticks: if quotes {
            Backticks::Off
        } else {
            options.backticks
        },
        dashes: if quotes { Dashes::Off } else { options.dashes },
        opening_quotes: options.opening_quotes.clone(),
        closing_quotes: options.closing_quotes.clone(),
    }
}

// unist-util-visit's preorder with upstream's test: text and inline code, paragraphs,
// headings and table cells, unless their parent is a JSX text element that is a fragment or
// a `style` or `script`
fn collect_pieces<'n, 'a>(root: &'n mut Node<'a>) -> Vec<Piece<'n, 'a>> {
    let mut pieces = Vec::new();
    let mut stack: Vec<(&'n mut Node<'a>, bool)> = Vec::new();
    let allowed = admits(root);
    if let Some(children) = root.children_mut() {
        stack.extend(children.iter_mut().rev().map(|child| (child, allowed)));
    }
    while let Some((node, allowed)) = stack.pop() {
        match node {
            Node::Text(text) => {
                if allowed {
                    pieces.push(Piece::Text(text));
                }
            }
            Node::InlineCode(code) => {
                if allowed {
                    pieces.push(Piece::Code(code.value.encode_utf16().count()));
                }
            }
            node => {
                if allowed
                    && matches!(
                        node,
                        Node::Paragraph(_) | Node::Heading(_) | Node::TableCell(_)
                    )
                {
                    pieces.push(Piece::Block);
                }
                let admitted = admits(node);
                if let Some(children) = node.children_mut() {
                    stack
                        .extend(children.iter_mut().rev().map(|child| (child, admitted)));
                }
            }
        }
    }
    pieces
}

fn admits(parent: &Node<'_>) -> bool {
    match parent {
        Node::MdxJsxTextElement(element) => element
            .name
            .is_some_and(|name| name != "style" && name != "script"),
        _ => true,
    }
}
