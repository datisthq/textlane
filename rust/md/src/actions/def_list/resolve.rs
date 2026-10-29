use crate::models::def_list::DefListStart;
use crate::models::event::{ContentType, Event, Kind, Link, Point};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const TAB_SIZE: usize = 4;

// The lines of a paragraph that become terms: the paragraph's enter and exit, its text
// chunks, the first that is a term, and the block quote exit to move before them
struct Terms {
    paragraph: (usize, usize),
    chunks: Vec<usize>,
    first: usize,
    block_quote: Option<usize>,
}

/// Turn the paragraph lines before each new definition description into terms, then wrap
/// terms and descriptions in definition lists, joining a list to the one right before it.
pub(crate) fn resolve_def_list(tokenizer: &mut Tokenizer) {
    let mut starts =
        std::mem::take(&mut tokenizer.tokenize_state.def_list.starts).into_iter();
    // The exit of the last description of each list
    let mut ends: Vec<usize> = vec![];
    // The list of the last description at each depth of nested descriptions, and where the
    // part of it that its terms opened starts
    let mut last: Vec<Option<usize>> = vec![];
    let mut starts_at: Vec<Option<Point>> = vec![];
    let mut open: Vec<usize> = vec![];
    for index in 0..tokenizer.events.len() {
        let event = &tokenizer.events[index];
        if event.name != Name::DEF_LIST_DESCRIPTION {
            continue;
        }
        if event.kind == Kind::Exit {
            if let Some(list) = open.pop() {
                ends[list] = index;
            }
            extend_description(&mut tokenizer.events, index);
            continue;
        }
        let depth = open.len();
        if last.len() <= depth {
            last.resize(depth + 1, None);
            starts_at.resize(depth + 1, None);
        }
        let start = starts.next().unwrap_or_default();
        let list = match last[depth] {
            Some(list) if start.sibling => {
                // As mdast-util-definition-list's, a description ends where the next one's
                // prefix does
                let end = ends[list];
                if let Some(prefix) = tokenizer.events[index..].iter().find(|event| {
                    event.kind == Kind::Exit
                        && event.name == Name::DEF_LIST_DESCRIPTION_PREFIX
                }) {
                    tokenizer.events[end].point = prefix.point.clone();
                }
                list
            }
            previous => {
                let (list, first) =
                    open_list(tokenizer, index, &start, previous, &mut ends);
                starts_at[depth] = Some(first);
                list
            }
        };
        start_description(tokenizer, index, starts_at[depth].as_ref());
        last[depth] = Some(list);
        open.push(list);
    }
    for &end in &ends {
        let point = tokenizer.events[end].point.clone();
        let exit = create_event(Kind::Exit, Name::DEF_LIST, point, None);
        tokenizer.map.add(end + 1, 0, vec![exit]);
    }
    tokenizer
        .map
        .consume(&mut tokenizer.events, &tokenizer.parse_state.spare_events);
}

// Make the terms of the description at `index`, then open a list for them or join the
// list before, when only container prefixes and line endings are between them: micromark's
// descriptions take the blank lines and the unclosed fence's line ending that end them
fn open_list(
    tokenizer: &mut Tokenizer,
    index: usize,
    start: &DefListStart,
    previous: Option<usize>,
    ends: &mut Vec<usize>,
) -> (usize, Point) {
    let terms = find_terms(&tokenizer.events, index, start);
    let mut removed = None;
    let first = match &terms {
        Some(terms) => {
            if terms.first == 0 {
                removed = Some(terms.paragraph.0);
            }
            shrink_paragraph(tokenizer, terms);
            terms.chunks[terms.first]
        }
        None => index,
    };
    let joined = previous.filter(|&list| {
        (ends[list] + 1..first).all(|between| {
            Some(between) == removed
                || matches!(
                    tokenizer.events[between].name,
                    Name::BLANK_LINE_ENDING
                        | Name::LINE_ENDING
                        | Name::BLOCK_QUOTE_PREFIX
                        | Name::BLOCK_QUOTE_MARKER
                        | Name::SPACE_OR_TAB
                )
        })
    });
    let list = joined.unwrap_or_else(|| {
        let point = tokenizer.events[first].point.clone();
        let enter = create_event(Kind::Enter, Name::DEF_LIST, point, None);
        tokenizer.map.add(first, 0, vec![enter]);
        ends.push(index);
        ends.len() - 1
    });
    if let Some(terms) = &terms {
        wrap_terms(tokenizer, terms);
    }
    (list, tokenizer.events[first].point.clone())
}

// A description starts after its prefix; one with nothing after it at the end of the
// document starts where the list its terms opened does, as mdast-util-definition-list's
// takes the start of the token after its prefix
fn start_description(tokenizer: &mut Tokenizer, enter: usize, list: Option<&Point>) {
    let events = &tokenizer.events;
    let Some(prefix) = events[enter..].iter().position(|event| {
        event.kind == Kind::Exit && event.name == Name::DEF_LIST_DESCRIPTION_PREFIX
    }) else {
        return;
    };
    let prefix = enter + prefix;
    let point = &events[prefix].point;
    let empty = events.get(prefix + 1).is_some_and(|event| {
        event.kind == Kind::Exit && event.name == Name::DEF_LIST_DESCRIPTION
    }) && !tokenizer.parse_state.bytes[point.index..]
        .iter()
        .any(|byte| matches!(byte, b'\n' | b'\r'));
    let start = list.filter(|_| empty).unwrap_or(point).clone();
    tokenizer.events[enter].point = start;
}

// micromark's description takes the line endings and blank lines after it, which ours
// leaves out, past the exits of containers closing with it
fn extend_description(events: &mut [Event], exit: usize) {
    let mut index = exit + 1;
    let mut end = None;
    // A blank line ends a block quote, so one after its exit is not the description's
    let mut quoted = false;
    while let Some(event) = events.get(index) {
        match (event.kind, event.name) {
            (_, Name::BLANK_LINE_ENDING) if quoted => break,
            (Kind::Exit, Name::BLOCK_QUOTE) => quoted = true,
            (Kind::Exit, Name::BLANK_LINE_ENDING | Name::LINE_ENDING) => {
                end = Some(index)
            }
            (_, Name::BLANK_LINE_ENDING | Name::LINE_ENDING | Name::SPACE_OR_TAB)
            | (
                Kind::Exit,
                Name::DEF_LIST_DESCRIPTION
                | Name::GFM_FOOTNOTE_DEFINITION
                | Name::LIST_ITEM,
            ) => {}
            _ => break,
        }
        index += 1;
    }
    if let Some(end) = end {
        let point = events[end].point.clone();
        for event in &mut events[exit..end] {
            if event.kind == Kind::Exit && event.name != Name::LINE_ENDING {
                event.point = point.clone();
            }
        }
    }
}

fn find_terms(
    events: &[Event],
    description: usize,
    start: &DefListStart,
) -> Option<Terms> {
    let from = start.terms?;
    let mut index = description;
    let mut block_quote = None;
    let exit = loop {
        index = index.checked_sub(1)?;
        let event = &events[index];
        if event.point.index < from {
            return None;
        }
        match (event.kind, event.name) {
            (Kind::Exit, Name::PARAGRAPH) => break index,
            (Kind::Exit, Name::BLOCK_QUOTE)
                if start.block_quote && block_quote.is_none() =>
            {
                block_quote = Some(index);
            }
            _ => {}
        }
    };
    let mut chunks = vec![];
    let enter = loop {
        index = index.checked_sub(1)?;
        let event = &events[index];
        if event.name == Name::PARAGRAPH {
            break index;
        }
        if event.kind == Kind::Enter && event.name == Name::DATA && event.link.is_some() {
            chunks.push(index);
        }
    };
    chunks.reverse();
    let first = chunks
        .iter()
        .position(|&chunk| events[chunk].point.index >= from)?;
    Some(Terms {
        paragraph: (enter, exit),
        chunks,
        first,
        block_quote,
    })
}

// The paragraph keeps the lines before the terms, or goes; a block quote closed by the
// list ends before its lazy lines that became terms
fn shrink_paragraph(tokenizer: &mut Tokenizer, terms: &Terms) {
    let (enter, exit) = terms.paragraph;
    let first = terms.chunks[terms.first];
    tokenizer.map.add(exit, 1, vec![]);
    if terms.first == 0 {
        tokenizer.map.add(enter, 1, vec![]);
    } else {
        let last = terms.chunks[terms.first - 1];
        if let Some(link) = tokenizer.events[last].link.as_mut() {
            link.next = None;
        }
        let mut paragraph_exit = tokenizer.events[exit].clone();
        paragraph_exit.point = tokenizer.events[last + 1].point.clone();
        tokenizer.map.add(last + 2, 0, vec![paragraph_exit]);
    }
    if let Some(index) = terms.block_quote {
        let mut block_quote_exit = tokenizer.events[index].clone();
        block_quote_exit.point = tokenizer.events[first].point.clone();
        tokenizer.map.add(index, 1, vec![]);
        tokenizer.map.add(first, 0, vec![block_quote_exit]);
    }
}

// Each line becomes a term of its own, its text no longer joined to the lines around it
fn wrap_terms(tokenizer: &mut Tokenizer, terms: &Terms) {
    let bytes = tokenizer.parse_state.bytes;
    for (position, &chunk) in terms.chunks.iter().enumerate().skip(terms.first) {
        let start = tokenizer.events[chunk].point.clone();
        let end = tokenizer.events[chunk + 1].point.clone();
        let unlinked = Link {
            previous: None,
            next: None,
            content: ContentType::Text,
        };
        let term = create_event(Kind::Enter, Name::DEF_LIST_TERM, start.clone(), None);
        tokenizer.map.add(chunk, 0, vec![term]);
        // micromark's flow takes the indent of a paragraph's first line only: later lines
        // keep theirs, which text would trim at its start
        let middle = (position > 0)
            .then(|| find_indent_end(bytes, &start, &end))
            .flatten();
        if let Some(middle) = middle {
            let indent = vec![
                create_event(Kind::Enter, Name::DATA, start, None),
                create_event(Kind::Exit, Name::DATA, middle.clone(), None),
                create_event(Kind::Enter, Name::DATA, middle, Some(unlinked)),
            ];
            tokenizer.map.add(chunk, 1, indent);
        } else {
            tokenizer.events[chunk].link = Some(unlinked);
        }
        // The last line's term takes its line ending, after the paragraph and the block
        // quote exit that moves
        let mut line_ending = chunk + 3;
        if terms.block_quote == Some(line_ending) {
            line_ending += 1;
        }
        let (at, end) = match tokenizer.events.get(line_ending) {
            Some(event)
                if position + 1 == terms.chunks.len()
                    && event.kind == Kind::Enter
                    && event.name == Name::LINE_ENDING =>
            {
                let end = tokenizer.events[line_ending + 1].point.clone();
                (line_ending + 2, end)
            }
            _ => (chunk + 2, end),
        };
        let term = create_event(Kind::Exit, Name::DEF_LIST_TERM, end, None);
        tokenizer.map.add(at, 0, vec![term]);
    }
}

// Where the spaces and tabs a chunk starts with end, unless that is its end
fn find_indent_end(bytes: &[u8], start: &Point, end: &Point) -> Option<Point> {
    let mut point = start.clone();
    if point.vs > 0 {
        point.column += TAB_SIZE - (point.column - 1) % TAB_SIZE;
        point.index += 1;
        point.vs = 0;
    }
    while point.index < end.index {
        match bytes[point.index] {
            b' ' => point.column += 1,
            b'\t' => point.column += TAB_SIZE - (point.column - 1) % TAB_SIZE,
            _ => break,
        }
        point.index += 1;
    }
    (point.index > start.index && point.index < end.index).then_some(point)
}

fn create_event(kind: Kind, name: Name, point: Point, link: Option<Link>) -> Event {
    Event {
        kind,
        name,
        point,
        link,
    }
}
