use crate::helpers::skip::skip_opt_back;
use crate::models::construct::State;
use crate::models::event::{ContentType, Event, Kind, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::content::tokenize_content_definition_before;
use crate::utils::tokenizer::document::tokenize_document;
use crate::utils::tokenizer::driver::Tokenizer;
use crate::utils::tokenizer::edit_map::EditMap;
use crate::utils::tokenizer::preprocess::ParseState;
use crate::utils::tokenizer::string::tokenize_string;
use crate::utils::tokenizer::text::tokenize_text;

/// What one round of tokenizing found; not `done` while linked content is left to tokenize.
#[derive(Debug)]
pub(crate) struct Subresult {
    pub(crate) done: bool,
    pub(crate) definitions: Vec<String>,
}

/// Link the void event before `index` to the one at `index`, two events back.
pub(crate) fn link(events: &mut [Event], index: usize) {
    link_to(events, index - 2, index);
}

pub(crate) fn link_to(events: &mut [Event], previous: usize, next: usize) {
    if let Some(link) = events[previous].link.as_mut() {
        link.next = Link::at(next);
    }
    if let Some(link) = events[next].link.as_mut() {
        link.previous = Link::at(previous);
    }
}

/// Tokenize every chain of linked content, optionally of one content type only.
pub(crate) fn subtokenize(
    events: &mut Vec<Event>,
    parse_state: &ParseState,
    filter: Option<ContentType>,
) -> Subresult {
    let mut map = EditMap::new();
    let mut value = Subresult {
        done: true,
        definitions: vec![],
    };
    let mut acc = (0, 0);
    let mut index = 0;
    while index < events.len() {
        let event = &events[index];
        if let Some(link) = &event.link
            && link.previous.is_none()
            && filter.is_none_or(|filter| link.content == filter)
        {
            let content = link.content;
            let mut tokenizer = Tokenizer::new(event.point, parse_state);
            let mut state = State::Next(match link.content {
                ContentType::Content => tokenize_content_definition_before,
                ContentType::String => tokenize_string,
                ContentType::Document => tokenize_document,
                _ => tokenize_text,
            });
            tokenizer.tokenize_state.text_trailing =
                link.content == ContentType::TextTrailing;
            tokenizer.tokenize_state.task_list_check_allowed =
                parse_state.gfm.is_some() && starts_list_item(events, index);
            let mut link_index = Some(index);
            while let Some(index) = link_index {
                let enter = &events[index];
                let current = enter.link.as_ref();
                if current.is_some_and(|link| link.previous.is_some()) {
                    tokenizer.define_skip(enter.point);
                }
                let end = &events[index + 1].point;
                state = tokenizer.push(
                    (enter.point.index as usize, enter.point.vs as usize),
                    (end.index as usize, end.vs as usize),
                    state,
                );
                link_index = current.and_then(Link::next);
            }
            let mut result = tokenizer.flush(state, true);
            if parse_state.failed.get() {
                return value;
            }
            // Strings depend on nothing found later: tokenized in the child's few events,
            // they save the document one more pass over all of its own
            if content != ContentType::String {
                subtokenize(
                    &mut tokenizer.events,
                    parse_state,
                    Some(ContentType::String),
                );
                if parse_state.failed.get() {
                    return value;
                }
            }
            value.definitions.append(&mut result.definitions);
            value.done = false;
            acc = divide_events(&mut map, events, index, &mut tokenizer.events, acc);
        }
        index += 1;
    }
    map.consume(events);
    value
}

// Text that opens a list item's first paragraph, as micromark's first content of an item:
// after one blank line ending the prefix, and after definitions in the same content
fn starts_list_item(events: &[Event], index: usize) -> bool {
    let is_exit =
        |at: usize, name| events[at].kind == Kind::Exit && events[at].name == name;
    if index < 3
        || events[index - 1].kind != Kind::Enter
        || events[index - 1].name != Name::PARAGRAPH
    {
        return false;
    }
    let mut at = skip_opt_back(events, index - 2, &[Name::SPACE_OR_TAB]);
    while is_exit(at, Name::LINE_ENDING) {
        at = skip_opt_back(events, at, &[Name::LINE_ENDING]);
        at = skip_opt_back(events, at, &[Name::SPACE_OR_TAB]);
        if !is_exit(at, Name::DEFINITION) {
            return false;
        }
        at = skip_opt_back(events, at, &[Name::DEFINITION]);
        at = skip_opt_back(events, at, &[Name::SPACE_OR_TAB]);
    }
    if is_exit(at, Name::BLANK_LINE_ENDING) {
        at = skip_opt_back(events, at, &[Name::BLANK_LINE_ENDING]);
    }
    is_exit(at, Name::LIST_ITEM_PREFIX)
}

/// Replace each linked chunk of `events` by the child events that fall inside it.
pub(crate) fn divide_events(
    map: &mut EditMap,
    events: &[Event],
    mut link_index: usize,
    child_events: &mut Vec<Event>,
    acc_before: (usize, usize),
) -> (usize, usize) {
    let mut child_index = 0;
    let mut slices = vec![];
    let mut slice_start = 0;
    let mut old_prev: Option<usize> = None;
    let len = child_events.len();
    while child_index < len {
        let current = &child_events[child_index].point;
        let end = &events[link_index + 1].point;
        // Inside the next slice
        if current.index > end.index
            || (current.index == end.index && current.vs > end.vs)
        {
            slices.push((link_index, slice_start));
            slice_start = child_index;
            link_index = events[link_index]
                .link
                .as_ref()
                .and_then(Link::next)
                .unwrap_or(link_index);
        }
        let has_previous = child_events[child_index]
            .link
            .as_ref()
            .is_some_and(|link| link.previous.is_some());
        if has_previous && let Some(old_prev) = old_prev {
            let new_link = if slices.is_empty() {
                old_prev + link_index + 2
            } else {
                old_prev + link_index - (slices.len() - 1) * 2
            };
            if let Some(link) = child_events[old_prev].link.as_mut() {
                link.next = Link::at(new_link + acc_before.1 - acc_before.0);
            }
        }
        let next = child_events[child_index].link.as_ref().and_then(Link::next);
        if let Some(next) = next
            && let Some(link) = child_events[next].link.as_mut()
        {
            old_prev = link.previous();
            link.previous = link.previous().and_then(|previous| {
                Link::at(
                    previous + link_index - (slices.len() * 2) + acc_before.1
                        - acc_before.0,
                )
            });
        }
        child_index += 1;
    }
    if !child_events.is_empty() {
        slices.push((link_index, slice_start));
    }
    let base = map.pool(child_events);
    for (index, &(at, start)) in slices.iter().enumerate() {
        let end = slices.get(index + 1).map_or(len, |next| next.1);
        map.add_pooled(at, 2, base + start..base + end);
    }
    (acc_before.0 + (slices.len() * 2), acc_before.1 + len)
}
