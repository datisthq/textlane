use crate::actions::def_list::track::has_line_events;
use crate::helpers::content::{find_chunks, find_paragraph_start};
use crate::models::def_list::DefListStart;
use crate::models::event::{Event, Kind};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::{Container, Tokenizer};
use crate::utils::tokenizer::preprocess::ParseState;

const TAB_SIZE: usize = 4;

/// What a line starting with `:` may open, as micromark's extension checks it first.
#[derive(Debug)]
pub(crate) enum DefListCheck {
    No,
    /// Another description of the list whose description did not continue.
    Sibling,
    /// A new list, once the flow and the `closing` containers that did not continue close;
    /// `block_quote` when that is one block quote and nothing indents the `:`.
    New {
        closing: usize,
        block_quote: bool,
    },
}

/// Whether a line may open a definition description: a `:` with whitespace after it, after
/// a paragraph (or a GFM table row, which ends instead).
pub(crate) fn check_def_list(tokenizer: &mut Tokenizer) -> DefListCheck {
    if !tokenizer.parse_state.definition_list || tokenizer.tokenize_state.document_opened
    {
        return DefListCheck::No;
    }
    let Some(indent) = find_marker(tokenizer) else {
        return DefListCheck::No;
    };
    let state = &tokenizer.tokenize_state;
    let continued = state.document_continued;
    let stack = &state.document_container_stack;
    let Some(child) = state.document_child.as_deref() else {
        return DefListCheck::No;
    };
    // micromark marks a description loose by the event right before its prefix: the
    // blank line, unless a container prefix or indent comes between
    let blank_before = child
        .events
        .last()
        .is_some_and(|event| event.name == Name::BLANK_LINE_ENDING);
    let plain = indent == 0 && !has_line_events(tokenizer);
    if let Some(container) = stack.get(continued)
        && container.kind == Container::DefList
    {
        let spread = container.blank_last
            || (plain && continued + 1 == stack.len() && blank_before);
        let track = &mut tokenizer.tokenize_state.def_list;
        track.pending = DefListStart {
            sibling: true,
            ..DefListStart::default()
        };
        track.pending_spread = spread;
        return DefListCheck::Sibling;
    }
    if state.def_list.line_boundary {
        return DefListCheck::No;
    }
    let Some((index, blanks)) = find_flow_tail(&child.events) else {
        return DefListCheck::No;
    };
    let passes = match child.events[index].name {
        Name::CONTENT if blanks == 0 => true,
        Name::GFM_TABLE_HEAD | Name::GFM_TABLE_ROW => blanks == 0,
        _ => find_term_paragraph(
            tokenizer.parse_state,
            &child.events,
            index,
            state.def_list.run_start.max(state.def_list.flow_start),
        )
        .is_some(),
    };
    if !passes {
        return DefListCheck::No;
    }
    let closing = stack.len() - continued;
    // micromark sees a closed block quote only as the last event, before the indent's
    let block_quote =
        closing == 1 && indent == 0 && stack[continued].kind == Container::BlockQuote;
    tokenizer.tokenize_state.def_list.pending_spread = plain && blank_before;
    DefListCheck::New {
        closing,
        block_quote,
    }
}

/// Whether a new definition list opens once the flow and the containers before it closed:
/// the flow must end in a paragraph, out of a block quote only through its lazy lines.
pub(crate) fn confirm_def_list(
    tokenizer: &mut Tokenizer,
    closing: usize,
    block_quote: bool,
) -> bool {
    if closing > 1 || (closing == 1 && !block_quote) {
        return false;
    }
    let state = &tokenizer.tokenize_state;
    let Some(child) = state.document_child.as_deref() else {
        return false;
    };
    let Some((index, blanks)) = find_flow_tail(&child.events) else {
        return false;
    };
    let track = &state.def_list;
    let from = track.run_start.max(track.flow_start);
    let paragraph =
        find_term_paragraph(tokenizer.parse_state, &child.events, index, from);
    // The flow closed here: content after this line does not join content before it
    tokenizer.tokenize_state.def_list.flow_start = tokenizer.point.index as usize;
    let track = &tokenizer.tokenize_state.def_list;
    let Some(paragraph) = paragraph else {
        return false;
    };
    let mut terms = Some(paragraph.max(track.pure_start));
    if block_quote {
        let lazy_end = if blanks == 0 {
            track.lazy_start.is_some()
        } else {
            track.lazy_before
        };
        if !lazy_end {
            return false;
        }
        terms = track
            .lazy_start
            .and_then(|lazy| terms.map(|start| start.max(lazy)));
        // Without a lazy last line, micromark's list starts right before the description
        if track.lazy_start.is_none() {
            tokenizer.tokenize_state.def_list.pending_spread = false;
        }
    }
    tokenizer.tokenize_state.def_list.pending = DefListStart {
        terms,
        sibling: false,
        block_quote,
    };
    true
}

// Up to three columns of indent, `:`, then whitespace or the end of the line
fn find_marker(tokenizer: &Tokenizer) -> Option<usize> {
    let bytes = tokenizer.parse_state.bytes;
    let point = &tokenizer.point;
    let mut index = point.index as usize;
    let mut column = point.column as usize - 1;
    let mut size = 0;
    // Inside a tab, its rest is indent
    if point.vs > 0 {
        size = TAB_SIZE - column % TAB_SIZE;
        column += size;
        index += 1;
    }
    while let Some(&byte @ (b' ' | b'\t')) = bytes.get(index) {
        let width = if byte == b' ' {
            1
        } else {
            TAB_SIZE - column % TAB_SIZE
        };
        size += width;
        column += width;
        index += 1;
        if size >= TAB_SIZE {
            return None;
        }
    }
    let after = bytes.get(index + 1);
    (bytes.get(index) == Some(&b':')
        && after.is_none_or(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r')))
    .then_some(size)
}

// The last flow token, past line endings, whitespace and at most one blank line, and how
// many blank lines came after it
fn find_flow_tail(events: &[Event]) -> Option<(usize, usize)> {
    let mut blanks = 0;
    let mut index = events.len();
    while index > 0 {
        index -= 1;
        let event = &events[index];
        match event.name {
            Name::LINE_ENDING | Name::SPACE_OR_TAB => {}
            Name::BLANK_LINE_ENDING => {
                if event.kind == Kind::Exit {
                    blanks += 1;
                    if blanks > 1 {
                        return None;
                    }
                }
            }
            _ => return Some((index, blanks)),
        }
    }
    None
}

// Where the paragraph that ends the flow at `index` starts, if at or after `from`. Setext
// underlines are tokenized as such until resolved: one with no open paragraph above it is
// a paragraph itself, or a thematic break when it is three dashes or more
fn find_term_paragraph(
    parse_state: &ParseState,
    events: &[Event],
    index: usize,
    from: usize,
) -> Option<usize> {
    let chunks = (events[index].name == Name::CONTENT)
        .then(|| find_chunks(events, index, from))
        .flatten();
    let mut before = match &chunks {
        Some(chunks) => chunks[0],
        None => index + 1,
    };
    let mut underlines = vec![];
    let content = loop {
        let mut at = before;
        while at > 0
            && matches!(events[at - 1].name, Name::LINE_ENDING | Name::SPACE_OR_TAB)
        {
            at -= 1;
        }
        let Some(at) = at.checked_sub(1) else {
            break None;
        };
        if events[at].name != Name::HEADING_SETEXT_UNDERLINE {
            break (events[at].name == Name::CONTENT).then_some(at);
        }
        let enter = (0..at).rev().find(|&enter| {
            events[enter].kind == Kind::Enter
                && events[enter].name == Name::HEADING_SETEXT_UNDERLINE
        })?;
        // Before `from`, it is across a container boundary
        if events[enter].point.index < from as u32 {
            break None;
        }
        before = enter;
        underlines.push(before);
    };
    if chunks.is_none() && underlines.is_empty() {
        return None;
    }
    let mut open = content
        .and_then(|exit| find_chunks(events, exit, from))
        .and_then(|chunks| find_paragraph_start(parse_state, events, &chunks))
        .is_some();
    let mut start = None;
    for &enter in underlines.iter().rev() {
        if open || is_dash_break(parse_state.bytes, events, enter) {
            open = false;
            start = None;
        } else {
            open = true;
            start = Some(events[enter].point.index);
        }
    }
    match (start, chunks) {
        (Some(start), _) => (start >= from as u32).then_some(start as usize),
        (None, Some(chunks)) => find_paragraph_start(parse_state, events, &chunks),
        (None, None) => None,
    }
}

fn is_dash_break(bytes: &[u8], events: &[Event], enter: usize) -> bool {
    let Some(sequence) = events[enter..]
        .iter()
        .position(|event| event.name == Name::HEADING_SETEXT_UNDERLINE_SEQUENCE)
    else {
        return false;
    };
    let start = events[enter + sequence].point.index;
    let end = events[enter + sequence + 1].point.index;
    end - start >= 3 && bytes.get(start as usize) == Some(&b'-')
}
