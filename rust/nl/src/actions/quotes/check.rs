use textlane_core::Position;
use textlane_core::nlcst::Node;

use crate::actions::nlcst::stringify::stringify_nlcst;
use crate::helpers::node::is_decade;
use crate::models::quotes::QuoteStyle;
use crate::models::report::Report;

/// What `nl.quotes` checks against, each marker in UTF-16 code units as upstream indexes
/// them.
pub(crate) struct QuoteMarkers {
    pub preferred: QuoteStyle,
    pub smart: Vec<Vec<u16>>,
    pub straight: Vec<Vec<u16>>,
}

/// Report quotes and apostrophes that are not the preferred ones at their level of nesting,
/// as retext-quotes does: every paragraph starts with no quote open.
pub(crate) fn check_quotes<'a>(
    tree: &Node<'a>,
    markers: &QuoteMarkers,
    reports: &mut Vec<Report<'a>>,
) {
    let mut stack = vec![tree];
    while let Some(node) = stack.pop() {
        if matches!(node, Node::Paragraph(_)) {
            check_paragraph(node, markers, reports);
        } else if let Some(children) = node.children() {
            stack.extend(children.iter().rev());
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Open,
    Close,
    Apostrophe,
}

#[derive(Clone, Copy)]
struct Marker<'s> {
    marker: &'s [u16],
    style: QuoteStyle,
    kind: Option<Kind>,
}

fn check_paragraph<'a>(
    paragraph: &Node<'a>,
    markers: &QuoteMarkers,
    reports: &mut Vec<Report<'a>>,
) {
    let mut open: Vec<Marker<'_>> = Vec::new();
    let mut frames: Vec<(&Node<'a>, usize)> = vec![(paragraph, 0)];
    while let Some((parent, index)) = frames.last_mut() {
        let parent = *parent;
        let Some(node) = parent.children().and_then(|children| children.get(*index))
        else {
            frames.pop();
            continue;
        };
        let position = *index;
        *index += 1;
        if let Node::Punctuation(literal) = node {
            let place = (parent, position, node.position().copied());
            check_mark(literal.value, place, markers, &mut open, reports);
        }
        if node.children().is_some() {
            frames.push((node, 0));
        }
    }
}

// A punctuation mark's parent, its index there and its position
type Place<'n, 'a> = (&'n Node<'a>, usize, Option<Position<'a>>);

fn check_mark<'a, 's>(
    value: &str,
    (parent, index, position): Place<'_, 'a>,
    markers: &'s QuoteMarkers,
    open: &mut Vec<Marker<'s>>,
    reports: &mut Vec<Report<'a>>,
) {
    let actual: Vec<u16> = value.encode_utf16().collect();
    let Some(mut marker) = find_marker(&actual, &markers.straight, QuoteStyle::Straight)
        .or_else(|| find_marker(&actual, &markers.smart, QuoteStyle::Smart))
    else {
        return;
    };
    let quote_like = value == "'" || value == "’";
    if quote_like || marker.kind.is_none() {
        marker.kind = Some(infer_kind(open, &marker, value, parent, index));
    }
    if marker.kind == Some(Kind::Open) {
        open.push(marker);
    }
    let smart = markers.preferred == QuoteStyle::Smart;
    let expected: Vec<u16> = if marker.kind == Some(Kind::Apostrophe) {
        (if smart { "’" } else { "'" }).encode_utf16().collect()
    } else {
        // A loose closing quote looks like an apostrophe
        if open.is_empty() && quote_like {
            return;
        }
        let list = if smart {
            &markers.smart
        } else {
            &markers.straight
        };
        let Some(level) = list.get(open.len().saturating_sub(1) % list.len().max(1))
        else {
            return;
        };
        if level.len() > 1 {
            read_unit(level, usize::from(marker.kind != Some(Kind::Open))).to_vec()
        } else {
            level.clone()
        }
    };
    if marker.kind == Some(Kind::Close) {
        open.pop();
    }
    if actual == expected {
        return;
    }
    let name = if marker.kind == Some(Kind::Apostrophe) {
        "apostrophe"
    } else {
        "quote"
    };
    let expected = String::from_utf16_lossy(&expected);
    let reason = if markers.preferred == marker.style {
        format!("Unexpected `{value}` at this level of nesting, expected `{expected}`")
    } else {
        let other = if smart { "straight" } else { "smart" };
        format!("Unexpected {other} {name} `{value}`, expected `{expected}`")
    };
    reports.push(Report {
        reason,
        rule: name.to_string(),
        source: "retext-quotes",
        position,
    });
}

// A marker matches by its first code unit, or as a closing quote by its second, as upstream's
// `charAt` comparisons do
fn find_marker<'s>(
    actual: &[u16],
    list: &'s [Vec<u16>],
    style: QuoteStyle,
) -> Option<Marker<'s>> {
    list.iter().find_map(|marker| {
        let both = marker.len() > 1;
        let kind = if read_unit(marker, 0) == actual {
            both.then_some(Kind::Open)
        } else if both && read_unit(marker, 1) == actual {
            Some(Kind::Close)
        } else {
            return None;
        };
        Some(Marker {
            marker,
            style,
            kind,
        })
    })
}

fn read_unit(units: &[u16], index: usize) -> &[u16] {
    units.get(index..index + 1).unwrap_or(&[])
}

fn infer_kind(
    open: &[Marker<'_>],
    marker: &Marker<'_>,
    value: &str,
    parent: &Node<'_>,
    index: usize,
) -> Kind {
    let siblings = parent.children().map_or(&[][..], |children| &children[..]);
    let previous = index.checked_sub(1).and_then(|index| siblings.get(index));
    let next = siblings.get(index + 1);
    let opens = open.last().is_none_or(|last| last.marker != marker.marker);
    if value == "'" || value == "’" {
        if matches!(parent, Node::Word(_)) {
            return Kind::Apostrophe;
        }
        if let Some(previous @ (Node::Source(_) | Node::Word(_))) = previous {
            // `Mr. Jones' golf clubs` against `'Mr. Jones' golf clubs`
            let before = stringify_nlcst(previous);
            let plural = before.ends_with(['s', 'S']);
            return if plural && opens {
                Kind::Apostrophe
            } else {
                Kind::Close
            };
        }
        if let Some(next @ Node::Word(_)) = next {
            return if is_decade(&stringify_nlcst(next)) {
                Kind::Apostrophe
            } else {
                Kind::Open
            };
        }
    }
    if opens { Kind::Open } else { Kind::Close }
}
