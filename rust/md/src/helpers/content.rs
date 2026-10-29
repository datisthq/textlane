use crate::models::construct::State;
use crate::models::event::{Event, Kind};
use crate::models::token::Name;
use crate::utils::tokenizer::content::tokenize_content_definition_before;
use crate::utils::tokenizer::driver::Tokenizer;
use crate::utils::tokenizer::preprocess::ParseState;

/// The chunks of the content ending at `exit`, first to last, from `from` at the earliest:
/// the flow joins lines across container boundaries, which micromark's does not.
pub(crate) fn find_chunks(
    events: &[Event],
    exit: usize,
    from: usize,
) -> Option<Vec<usize>> {
    let mut chunks = vec![exit.checked_sub(1)?];
    if events[chunks[0]].point.index < from as u32 {
        return None;
    }
    while let Some(&enter) = chunks.last() {
        let mut index = enter;
        while index > 0 && events[index - 1].name == Name::SPACE_OR_TAB {
            index -= 1;
        }
        if index < 4
            || events[index - 1].name != Name::LINE_ENDING
            || events[index - 3].name != Name::CONTENT
            || events[index - 3].kind != Kind::Exit
            || events[index - 4].point.index < from as u32
        {
            break;
        }
        chunks.push(index - 4);
    }
    chunks.reverse();
    Some(chunks)
}

/// Where the paragraph of the content in `chunks` starts, after its definitions; `None`
/// when the content holds only definitions.
pub(crate) fn find_paragraph_start(
    parse_state: &ParseState,
    events: &[Event],
    chunks: &[usize],
) -> Option<usize> {
    let first = &events[chunks[0]].point;
    let bytes = parse_state.bytes;
    let mut start = first.index;
    while matches!(bytes.get(start as usize), Some(b' ' | b'\t')) {
        start += 1;
    }
    // Only a definition, which starts with `[`, comes before a paragraph
    if bytes.get(start as usize) != Some(&b'[') {
        return Some(first.index as usize);
    }
    let mut tokenizer = Tokenizer::new(*first, parse_state);
    let mut state = State::Next(tokenize_content_definition_before);
    for (position, &enter) in chunks.iter().enumerate() {
        let start = &events[enter].point;
        if position > 0 {
            tokenizer.define_skip(*start);
        }
        // Each chunk but the last takes the line ending after it, as content joins them
        let end = if position + 1 < chunks.len() {
            &events[enter + 3].point
        } else {
            &events[enter + 1].point
        };
        state = tokenizer.push(
            (start.index as usize, start.vs as usize),
            (end.index as usize, end.vs as usize),
            state,
        );
    }
    tokenizer.flush(state, false);
    tokenizer
        .events
        .iter()
        .find(|event| event.kind == Kind::Enter && event.name == Name::PARAGRAPH)
        .map(|event| event.point.index as usize)
}
