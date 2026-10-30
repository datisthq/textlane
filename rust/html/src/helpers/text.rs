use std::borrow::Cow;

use textlane_core::hast::{Element, Node};

use crate::helpers::value::{find_value, is_truthy};

// Elements recurse once per level; past the red zone the stack grows on the heap
const STACK_RED_ZONE: usize = 64 * 1024;
const STACK_GROWTH: usize = 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Whitespace {
    Normal,
    Pre,
    PreWrap,
    Nowrap,
}

// What hast-util-to-text collects: text, or how many line endings may go between texts
enum Piece<'n> {
    Text(Cow<'n, str>),
    Breaks(i64),
}

/// The text an element renders, as hast-util-to-text's `toText` reads it.
pub(crate) fn read_text(element: &Element<'_>) -> String {
    collect_text_as(element, Whitespace::Normal)
}

/// The text of an element as `toText` reads it with `whitespace: 'pre'`.
#[cfg(feature = "katex")]
pub(crate) fn read_preformatted_text(element: &Element<'_>) -> String {
    collect_text_as(element, Whitespace::Pre)
}

fn collect_text_as(element: &Element<'_>, inherited: Whitespace) -> String {
    let block = is_block_or_caption(element.tag_name);
    let whitespace = infer_whitespace(element, inherited);
    let mut pieces = vec![];
    collect_children(&element.children, whitespace, block, block, &mut pieces);
    join_pieces(&pieces)
}

fn join_pieces(pieces: &[Piece<'_>]) -> String {
    let mut result = String::new();
    let mut count: Option<i64> = None;
    for piece in pieces {
        match piece {
            Piece::Breaks(value) => {
                if let Some(current) = count
                    && *value > current
                {
                    count = Some(*value);
                }
            }
            Piece::Text(value) if !value.is_empty() => {
                if let Some(current) = count
                    && current > -1
                {
                    match current {
                        0 => result.push(' '),
                        _ => (0..current).for_each(|_| result.push('\n')),
                    }
                }
                count = Some(-1);
                result.push_str(value);
            }
            Piece::Text(_) => {}
        }
    }
    result
}

// The children of a node; `before` and `after` are whether a break sits before the first
// and after the last
fn collect_children<'n>(
    children: &'n [Node<'_>],
    whitespace: Whitespace,
    before: bool,
    after: bool,
    pieces: &mut Vec<Piece<'n>>,
) {
    // Whether a row or cell has a sibling of its kind after it, as `findAfter` checks
    let mut later_row = vec![false; children.len()];
    let mut later_cell = vec![false; children.len()];
    let (mut row, mut cell) = (false, false);
    for (index, child) in children.iter().enumerate().rev() {
        later_row[index] = row;
        later_cell[index] = cell;
        if let Node::Element(element) = child {
            row |= element.tag_name == "tr";
            cell |= is_cell(element.tag_name);
        }
    }
    for (index, child) in children.iter().enumerate() {
        let break_before = if index == 0 { before } else { false };
        let break_after = match children.get(index + 1) {
            Some(next) => is_element(next, "br"),
            None => after,
        };
        match child {
            Node::Element(element) => {
                let later = (later_row[index], later_cell[index]);
                stacker::maybe_grow(STACK_RED_ZONE, STACK_GROWTH, || {
                    collect_element(element, later, whitespace, pieces);
                });
            }
            Node::Text(text) if whitespace == Whitespace::Normal => {
                collect_text(text.value, break_before, break_after, pieces);
            }
            Node::Text(text) => pieces.push(Piece::Text(Cow::Borrowed(text.value))),
            _ => {}
        }
    }
}

fn collect_element<'n>(
    element: &'n Element<'_>,
    (later_row, later_cell): (bool, bool),
    whitespace: Whitespace,
    pieces: &mut Vec<Piece<'n>>,
) {
    let whitespace = infer_whitespace(element, whitespace);
    if is_not_rendered(element) {
        return;
    }
    let tag_name = element.tag_name;
    let (prefix, suffix) = if tag_name == "br" || (tag_name == "tr" && later_row) {
        (None, Some(Piece::Text(Cow::Borrowed("\n"))))
    } else if tag_name == "p" {
        (Some(2), Some(Piece::Breaks(2)))
    } else if is_block_or_caption(tag_name) {
        (Some(1), Some(Piece::Breaks(1)))
    } else {
        (None, None)
    };
    if let Some(prefix) = prefix {
        pieces.push(Piece::Breaks(prefix));
    }
    collect_children(
        &element.children,
        whitespace,
        prefix.is_some(),
        suffix.is_some(),
        pieces,
    );
    if is_cell(tag_name) && later_cell {
        pieces.push(Piece::Text(Cow::Borrowed("\t")));
    }
    if let Some(suffix) = suffix {
        pieces.push(suffix);
    }
}

fn collect_text(value: &str, before: bool, after: bool, pieces: &mut Vec<Piece<'_>>) {
    let mut lines = vec![];
    let mut start = 0;
    loop {
        let end = value[start..]
            .find('\n')
            .map_or(value.len(), |end| start + end);
        let line: String = value[start..end]
            .chars()
            .filter(|character| !is_bidi_control(*character))
            .collect();
        lines.push(trim_and_collapse(
            &line,
            if start == 0 { before } else { true },
            if end == value.len() { after } else { true },
        ));
        if end == value.len() {
            break;
        }
        start = end + 1;
    }
    let mut join = false;
    let last = lines.len() - 1;
    for index in 0..lines.len() {
        let zero_width = lines[index].ends_with('\u{200B}')
            || (index < last && lines[index + 1].starts_with('\u{200B}'));
        if zero_width {
            pieces.push(Piece::Text(Cow::Owned(std::mem::take(&mut lines[index]))));
            join = false;
        } else if !lines[index].is_empty() {
            if join {
                pieces.push(Piece::Breaks(0));
            }
            pieces.push(Piece::Text(Cow::Owned(std::mem::take(&mut lines[index]))));
            join = true;
        } else if index == 0 || index == last {
            pieces.push(Piece::Breaks(0));
        }
    }
}

// Collapse runs of spaces and tabs into one space, dropping them at a break
fn trim_and_collapse(value: &str, before: bool, after: bool) -> String {
    let bytes = value.as_bytes();
    let mut result: Vec<&str> = vec![];
    let mut start = 0;
    let mut end = None;
    while start < bytes.len() {
        let found = bytes[start..]
            .iter()
            .position(|byte| matches!(byte, b'\t' | b' '))
            .map(|offset| start + offset);
        let current = found.unwrap_or(bytes.len());
        end = Some(current);
        if start == 0 && current == 0 && found.is_some() && !before {
            result.push("");
        }
        if start != current {
            result.push(&value[start..current]);
        }
        start = match found {
            Some(found) => {
                found
                    + bytes[found..]
                        .iter()
                        .take_while(|byte| matches!(byte, b'\t' | b' '))
                        .count()
            }
            None => current,
        };
    }
    if Some(start) != end && !after {
        result.push("");
    }
    result.join(" ")
}

fn infer_whitespace(element: &Element<'_>, inherited: Whitespace) -> Whitespace {
    match element.tag_name {
        "listing" | "plaintext" | "xmp" => Whitespace::Pre,
        "nobr" => Whitespace::Nowrap,
        "pre" if is_truthy(find_value(element, "wrap")) => Whitespace::PreWrap,
        "pre" => Whitespace::Pre,
        "td" | "th" if is_truthy(find_value(element, "noWrap")) => Whitespace::Nowrap,
        "textarea" => Whitespace::PreWrap,
        _ => inherited,
    }
}

fn is_not_rendered(element: &Element<'_>) -> bool {
    matches!(
        element.tag_name,
        "datalist"
            | "head"
            | "noembed"
            | "noframes"
            | "noscript"
            | "rp"
            | "script"
            | "style"
            | "template"
            | "title"
    ) || is_truthy(find_value(element, "hidden"))
        || (element.tag_name == "dialog" && !is_truthy(find_value(element, "open")))
}

// hast-util-to-text's list misspells `form` as `'form,'`, so a form is not a block
fn is_block_or_caption(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "body"
            | "caption"
            | "center"
            | "dd"
            | "dialog"
            | "dir"
            | "dl"
            | "dt"
            | "div"
            | "figure"
            | "figcaption"
            | "footer"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hgroup"
            | "hr"
            | "html"
            | "legend"
            | "li"
            | "listing"
            | "main"
            | "menu"
            | "nav"
            | "ol"
            | "p"
            | "plaintext"
            | "pre"
            | "section"
            | "ul"
            | "xmp"
    )
}

fn is_cell(tag_name: &str) -> bool {
    tag_name == "td" || tag_name == "th"
}

fn is_element(node: &Node<'_>, tag_name: &str) -> bool {
    matches!(node, Node::Element(element) if element.tag_name == tag_name)
}

fn is_bidi_control(character: char) -> bool {
    matches!(
        character,
        '\u{061C}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
    )
}
