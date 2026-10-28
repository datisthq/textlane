use crate::models::event::{ContentType, Event, Kind, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// Turn what lies between a heading's opening and closing sequences into its text, as
/// micromark does: `#`s inside, even right after the opening, are text.
pub(crate) fn resolve_heading_atx(tokenizer: &mut Tokenizer) {
    let mut index = 0;
    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];
        if event.kind == Kind::Enter && event.name == Name::HEADING_ATX {
            index = resolve_heading(tokenizer, index);
        }
        index += 1;
    }
    tokenizer.map.consume(&mut tokenizer.events);
}

// The heading's child tokens as (enter, exit) indexes; returns the heading's exit index
fn resolve_heading(tokenizer: &mut Tokenizer, enter: usize) -> usize {
    let events = &tokenizer.events;
    let mut tokens: Vec<(usize, usize)> = Vec::new();
    let mut index = enter + 1;
    while index < events.len() && events[index].name != Name::HEADING_ATX {
        let start = index;
        let mut depth = 0;
        loop {
            match events[index].kind {
                Kind::Enter => depth += 1,
                Kind::Exit => depth -= 1,
            }
            if depth == 0 {
                break;
            }
            index += 1;
        }
        tokens.push((start, index));
        index += 1;
    }
    let exit = index;
    let is = |token: usize, name: Name| events[tokens[token].0].name == name;
    // After the opening sequence and its whitespace, before a closing sequence that
    // whitespace separates and that whitespace may follow
    // The indent before the opening sequence is part of the heading here, unlike micromark
    let opening = (0..tokens.len()).find(|token| is(*token, Name::HEADING_ATX_SEQUENCE));
    let mut start = opening.map_or(tokens.len(), |opening| opening + 1);
    if start < tokens.len() && is(start, Name::SPACE_OR_TAB) {
        start += 1;
    }
    let Some(mut end) = tokens.len().checked_sub(1) else {
        return exit;
    };
    if end > start && is(end, Name::SPACE_OR_TAB) {
        end -= 1;
    }
    if end >= start
        && is(end, Name::HEADING_ATX_SEQUENCE)
        && (start == end || (end >= start + 2 && is(end - 1, Name::SPACE_OR_TAB)))
    {
        if start == end {
            return exit;
        }
        end -= 2;
    }
    if end < start {
        return exit;
    }
    let (text_enter, _) = tokens[start];
    let (_, text_exit) = tokens[end];
    let enter_point = events[text_enter].point.clone();
    let exit_point = events[text_exit].point.clone();
    let data = |kind, point| Event {
        kind,
        name: Name::DATA,
        point,
        link: (kind == Kind::Enter).then_some(Link {
            previous: None,
            next: None,
            content: ContentType::Text,
        }),
    };
    let text = |kind, point| Event {
        kind,
        name: Name::HEADING_ATX_TEXT,
        point,
        link: None,
    };
    tokenizer.map.add(
        text_enter,
        text_exit - text_enter + 1,
        vec![
            text(Kind::Enter, enter_point.clone()),
            data(Kind::Enter, enter_point),
            data(Kind::Exit, exit_point.clone()),
            text(Kind::Exit, exit_point),
        ],
    );
    exit
}
