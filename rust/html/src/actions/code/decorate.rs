use std::sync::LazyLock;

use regex::Regex;
use textlane_core::hast::{Element, Node, Property, PropertyItem, PropertyValue};
use textlane_core::{Allocator, ArenaVec, Error};

use crate::actions::code::transform::{configure_transformer, read_classes};
use crate::models::highlight::{CodeBlock, CodeToken};
use crate::models::options::{ConfiguredTransformer, HighlightTransformer};

/// A span of the code to mark, in UTF-16 units as Shiki's decorations count, and the
/// classes it adds.
pub(crate) struct Decoration {
    start: usize,
    end: usize,
    classes: Vec<String>,
}

static META_WORD: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"/((?:\\.|[^/])+)/").ok());
static ESCAPE: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"\\(.)").ok());

/// The decorations `metaWordHighlight` adds in its `preprocess`: each `/word/` of the meta
/// at every place it occurs, failing as Shiki does on ones that cross.
pub(crate) fn read_decorations(
    code: &str,
    meta: &str,
    transformers: &[HighlightTransformer],
) -> Result<Vec<Decoration>, Error> {
    let mut decorations = Vec::new();
    for transformer in transformers {
        let ConfiguredTransformer::MetaWordHighlight { class_name } =
            configure_transformer(transformer)
        else {
            continue;
        };
        if meta.is_empty() {
            continue;
        }
        let classes = read_classes(&class_name, "highlighted-word");
        for word in read_words(meta) {
            for start in find_all(code, &word) {
                let end = start + word.encode_utf16().count();
                decorations.push(Decoration {
                    start,
                    end,
                    classes: classes.clone(),
                });
            }
        }
    }
    check_intersections(&decorations)?;
    Ok(decorations)
}

/// Cut tokens at every decoration's start and end, as the decorations' `tokens` hook does,
/// so each decoration covers whole tokens.
pub(crate) fn split_tokens(
    block: &mut CodeBlock,
    code: &str,
    decorations: &[Decoration],
) {
    let mut breakpoints: Vec<usize> =
        decorations.iter().flat_map(|d| [d.start, d.end]).collect();
    breakpoints.sort_unstable();
    breakpoints.dedup();
    let mut line_start = 0;
    for (line, text) in block.lines.iter_mut().zip(code.split('\n')) {
        let mut offset = line_start;
        let mut tokens = Vec::with_capacity(line.tokens.len());
        for token in std::mem::take(&mut line.tokens).into_iter().flatten() {
            let length = token.text.encode_utf16().count();
            // `findFirstBreakpointAfter`: the breakpoints are sorted, so search, never scan
            let first = breakpoints.partition_point(|&point| point <= offset);
            let inside: Vec<usize> = breakpoints[first..]
                .iter()
                .take_while(|&&point| point < offset + length)
                .map(|point| point - offset)
                .collect();
            offset += length;
            if inside.is_empty() {
                tokens.push(Some(token));
                continue;
            }
            let units: Vec<u16> = token.text.encode_utf16().collect();
            let mut last = 0;
            for point in inside.into_iter().chain([length]) {
                tokens.push(Some(CodeToken {
                    text: String::from_utf16_lossy(&units[last..point]),
                    style: token.style.clone(),
                    classes: token.classes.clone(),
                }));
                last = point;
            }
        }
        line.tokens = tokens;
        line_start += text.encode_utf16().count() + 1;
    }
}

/// Apply decorations to the finished `<code>` children as the decorations' `code` hook
/// does: a decoration covering a whole line marks the line, one covering a single token
/// marks it, any other wraps what it covers in a `span`.
pub(crate) fn apply_decorations<'a>(
    children: &mut ArenaVec<'a, Node<'a>>,
    code: &str,
    decorations: &mut [Decoration],
    alloc: &'a Allocator,
) -> Result<(), Error> {
    let lengths: Vec<usize> = split_lines(code);
    let mut starts = Vec::with_capacity(lengths.len());
    let mut total = 0;
    for length in &lengths {
        starts.push(total);
        total += length;
    }
    let lines: Vec<usize> = (0..children.len())
        .filter(
            |&index| matches!(&children[index], Node::Element(e) if e.tag_name == "span"),
        )
        .collect();
    if lines.len() != lengths.len() {
        return Err(failed(&format!(
            "Number of lines in code element ({}) does not match the number of lines in the source ({}). Failed to apply decorations.",
            lines.len(),
            lengths.len()
        )));
    }
    decorations.sort_by(|a, b| b.start.cmp(&a.start).then(a.end.cmp(&b.end)));
    for decoration in decorations.iter() {
        let (start_line, start) = to_position(&starts, total, decoration.start);
        let (end_line, end) = to_position(&starts, total, decoration.end);
        if start_line != end_line {
            return Err(failed("A decoration spans lines, which a meta word cannot"));
        }
        let Some(Node::Element(line)) = children.get_mut(lines[start_line]) else {
            continue;
        };
        apply_section(line, start, end, &decoration.classes, alloc)?;
    }
    Ok(())
}

// `parseMetaHighlightWords`: each `/…/`, its escapes undone
fn read_words(meta: &str) -> Vec<String> {
    let (Some(word), Some(escape)) = (META_WORD.as_ref(), ESCAPE.as_ref()) else {
        return Vec::new();
    };
    word.captures_iter(meta)
        .filter_map(|captures| captures.get(1))
        .map(|found| escape.replace_all(found.as_str(), "$1").into_owned())
        .collect()
}

// `findAllSubstringIndexes`: occurrences that do not overlap, as UTF-16 offsets
fn find_all(code: &str, word: &str) -> Vec<usize> {
    let mut found = Vec::new();
    let mut cursor = 0;
    // UTF-16 counted from the last match on, so many matches stay linear
    let mut units = 0;
    while let Some(at) = code[cursor..].find(word) {
        let byte = cursor + at;
        units += code[cursor..byte].encode_utf16().count();
        found.push(units);
        let next = byte + word.len().max(1);
        if next > code.len() {
            break;
        }
        units += code[byte..next].encode_utf16().count();
        cursor = next;
    }
    found
}

// `verifyIntersections`: nested or touching decorations are fine, crossing ones fail
fn check_intersections(items: &[Decoration]) -> Result<(), Error> {
    let mut sorted: Vec<(usize, usize)> =
        items.iter().map(|d| (d.start, d.end)).collect();
    sorted.sort_unstable();
    // only neighbours in start order can cross: a later start inside an earlier span must
    // end inside it too
    let mut open: Vec<(usize, usize)> = Vec::new();
    for (start, end) in sorted {
        while open.last().is_some_and(|&(_, last_end)| last_end <= start) {
            open.pop();
        }
        if let Some(&(outer_start, outer_end)) = open.last()
            && end > outer_end
            && !(start == end || (outer_start == start && outer_end <= end))
        {
            return Err(failed("Decorations intersect."));
        }
        open.push((start, end));
    }
    Ok(())
}

// Each line's length with its line ending, as `createPositionConverter` keeps them
fn split_lines(code: &str) -> Vec<usize> {
    let mut lengths: Vec<usize> = code
        .split('\n')
        .map(|line| line.encode_utf16().count() + 1)
        .collect();
    if let Some(last) = lengths.last_mut() {
        *last -= 1;
    }
    lengths
}

// `indexToPos`: the line and the character inside it, the code's end on its last line
fn to_position(starts: &[usize], total: usize, index: usize) -> (usize, usize) {
    if index >= total {
        let last = starts.len().saturating_sub(1);
        return (last, index - starts.get(last).copied().unwrap_or(0));
    }
    let line = starts
        .partition_point(|&start| start <= index)
        .saturating_sub(1);
    (line, index - starts.get(line).copied().unwrap_or(0))
}

// `applyLineSection` for a decoration inside one line
fn apply_section<'a>(
    line: &mut Element<'a>,
    start: usize,
    end: usize,
    classes: &[String],
    alloc: &'a Allocator,
) -> Result<(), Error> {
    let mut start_index = (start == 0).then_some(0);
    let mut end_index = (end == 0).then_some(0);
    let mut length = 0;
    for (index, child) in line.children.iter().enumerate() {
        length += text_length(child);
        if start_index.is_none() && length == start {
            start_index = Some(index + 1);
        }
        if end_index.is_none() && length == end {
            end_index = Some(index + 1);
        }
        if start_index.is_some() && end_index.is_some() {
            break;
        }
    }
    let start_index =
        start_index.ok_or_else(|| failed("Failed to find start index for decoration"))?;
    let end_index =
        end_index.ok_or_else(|| failed("Failed to find end index for decoration"))?;
    let count = end_index.saturating_sub(start_index);
    if count == line.children.len() {
        add_class(&mut line.properties, classes, alloc);
        return Ok(());
    }
    if count == 1
        && let Some(Node::Element(token)) = line.children.get_mut(start_index)
    {
        add_class(&mut token.properties, classes, alloc);
        return Ok(());
    }
    let mut wrapped = ArenaVec::with_capacity_in(count, alloc);
    wrapped.extend(line.children.drain(start_index..end_index));
    let mut properties = ArenaVec::new_in(alloc);
    add_class(&mut properties, classes, alloc);
    line.children.insert(
        start_index,
        Node::Element(Element {
            tag_name: "span",
            properties,
            children: wrapped,
            position: None,
            data: None,
            content: None,
        }),
    );
    Ok(())
}

// hast-util-to-string's text length, in UTF-16 units as JavaScript counts it
fn text_length(node: &Node<'_>) -> usize {
    let mut length = 0;
    let mut stack = vec![node];
    while let Some(node) = stack.pop() {
        match node {
            Node::Text(text) => length += text.value.encode_utf16().count(),
            Node::Element(element) => stack.extend(element.children.iter()),
            _ => {}
        }
    }
    length
}

// `addClassToHast`: each name once, after the element's own
fn add_class<'a>(
    properties: &mut ArenaVec<'a, Property<'a>>,
    classes: &[String],
    alloc: &'a Allocator,
) {
    let index = match properties
        .iter()
        .position(|property| property.name == "className")
    {
        Some(index) => index,
        None => {
            properties.push(Property {
                name: "className",
                value: PropertyValue::List(ArenaVec::new_in(alloc)),
            });
            properties.len() - 1
        }
    };
    if let PropertyValue::List(items) = &mut properties[index].value {
        for class in classes {
            let present = items
                .iter()
                .any(|item| matches!(item, PropertyItem::String(n) if n == class));
            if !class.is_empty() && !present {
                items.push(PropertyItem::String(alloc.alloc_str(class)));
            }
        }
    }
}

fn failed(message: &str) -> Error {
    Error::new("highlight-failed", message)
}
