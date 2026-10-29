use crate::actions::block_quote::tokenize::{continue_block_quote, tokenize_block_quote};
use crate::actions::def_list::check::{DefListCheck, check_def_list, confirm_def_list};
use crate::actions::def_list::tokenize::{continue_def_list, tokenize_def_list};
use crate::actions::def_list::track::track_def_list;
use crate::actions::gfm_footnote_definition::tokenize::{
    continue_gfm_footnote_definition, tokenize_gfm_footnote_definition,
};
use crate::actions::list_item::tokenize::{continue_list_item, tokenize_list_item};
use crate::helpers::skip::{skip_opt_back, skip_to};
use crate::models::construct::{Resolver, State};
use crate::models::directive::CHUNK_DOCUMENT;
use crate::models::event::{ContentType, Event, Kind, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::{
    Container, ContainerState, TokenizeState, Tokenizer,
};
use crate::utils::tokenizer::flow::tokenize_flow;
use crate::utils::tokenizer::subtokenize::divide_events;

#[derive(Debug, PartialEq)]
enum Phase {
    After,
    Prefix,
    Eof,
}

/// The document content type: containers line by line, with flow inside them in a child
/// tokenizer.
pub(crate) fn tokenize_document(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.document_child = Some(Box::new(Tokenizer::new(
        tokenizer.point.clone(),
        tokenizer.parse_state,
    )));
    State::Retry(container_existing_before)
}

fn container_existing_before(tokenizer: &mut Tokenizer) -> State {
    // On an empty line, list items that did not start blank all continue without consuming
    // anything: skip them at once, as checking each costs the nesting depth per line
    if matches!(tokenizer.current, None | Some(b'\n')) {
        let continued = tokenizer.tokenize_state.document_continued;
        let end = find_blank_end(&mut tokenizer.tokenize_state, continued);
        tokenizer.tokenize_state.document_continued = end;
    }
    let state = &tokenizer.tokenize_state;
    let Some(container) = state.document_container_stack.get(state.document_continued)
    else {
        return State::Retry(container_new_before);
    };
    let continuation = match container.kind {
        Container::BlockQuote => continue_block_quote,
        Container::DefList => continue_def_list,
        Container::GfmFootnoteDefinition => continue_gfm_footnote_definition,
        Container::ListItem => continue_list_item,
    };
    tokenizer.tokenize_state.def_list.events_before = tokenizer.events.len();
    tokenizer.attempt(
        State::Next(container_existing_after),
        State::Next(container_new_before),
    );
    State::Retry(continuation)
}

fn container_existing_after(tokenizer: &mut Tokenizer) -> State {
    let state = &mut tokenizer.tokenize_state;
    // micromark's definition lists see a list item's or footnote's indent as a boundary
    if let Some(container) = state.document_container_stack.get(state.document_continued)
        && matches!(
            container.kind,
            Container::ListItem | Container::GfmFootnoteDefinition
        )
        && tokenizer.events.len() > state.def_list.events_before
    {
        state.def_list.line_boundary = true;
    }
    tokenizer.tokenize_state.document_continued += 1;
    State::Retry(container_existing_before)
}

fn container_new_before(tokenizer: &mut Tokenizer) -> State {
    let state = &tokenizer.tokenize_state;
    // Every container continued: a fenced code block (concrete) takes the line whole
    if state.document_continued == state.document_container_stack.len()
        && let Some(child) = state.document_child.as_ref()
    {
        // micromark decides `interrupt` once a line, before any container opens on it
        if !state.document_opened {
            tokenizer.interrupt = child.interrupt;
        }
        if child.concrete {
            return State::Retry(containers_after);
        }
    }
    // The new container goes where the next one to continue was, for its constructs to
    // store their state; it moves to the end once it starts
    push_container(tokenizer, Container::BlockQuote);
    tokenizer.attempt(
        State::Next(container_new_after),
        State::Next(container_new_before_not_block_quote),
    );
    State::Retry(tokenize_block_quote)
}

fn container_new_before_not_block_quote(tokenizer: &mut Tokenizer) -> State {
    let continued = tokenizer.tokenize_state.document_continued;
    tokenizer.tokenize_state.document_container_stack[continued] =
        new_container(Container::ListItem);
    tokenizer.attempt(
        State::Next(container_new_after),
        State::Next(container_new_before_not_list),
    );
    State::Retry(tokenize_list_item)
}

fn container_new_before_not_list(tokenizer: &mut Tokenizer) -> State {
    let continued = tokenizer.tokenize_state.document_continued;
    tokenizer.tokenize_state.document_container_stack[continued] =
        new_container(Container::GfmFootnoteDefinition);
    tokenizer.attempt(
        State::Next(container_new_after),
        State::Next(container_new_before_not_gfm_footnote_definition),
    );
    State::Retry(tokenize_gfm_footnote_definition)
}

fn container_new_before_not_gfm_footnote_definition(tokenizer: &mut Tokenizer) -> State {
    let continued = tokenizer.tokenize_state.document_continued;
    tokenizer
        .tokenize_state
        .document_container_stack
        .swap_remove(continued);
    match check_def_list(tokenizer) {
        DefListCheck::No => State::Retry(containers_after),
        DefListCheck::Sibling => State::Retry(def_list_before),
        // micromark's check passed: the flow and the containers that did not continue close
        // whether or not a list opens
        DefListCheck::New {
            closing,
            block_quote,
        } => {
            exit_containers(tokenizer, &Phase::Prefix);
            if let Some(child) = tokenizer.tokenize_state.document_child.as_mut() {
                child.pierce = true;
                child.interrupt = false;
            }
            if confirm_def_list(tokenizer, closing, block_quote) {
                State::Retry(def_list_before)
            } else {
                State::Retry(containers_after)
            }
        }
    }
}

fn def_list_before(tokenizer: &mut Tokenizer) -> State {
    push_container(tokenizer, Container::DefList);
    tokenizer.attempt(State::Next(container_new_after), State::Next(def_list_nok));
    State::Retry(tokenize_def_list)
}

fn def_list_nok(tokenizer: &mut Tokenizer) -> State {
    let continued = tokenizer.tokenize_state.document_continued;
    tokenizer
        .tokenize_state
        .document_container_stack
        .swap_remove(continued);
    State::Retry(containers_after)
}

fn container_new_after(tokenizer: &mut Tokenizer) -> State {
    let continued = tokenizer.tokenize_state.document_continued;
    let container = tokenizer
        .tokenize_state
        .document_container_stack
        .swap_remove(continued);
    // A new container closes the flow, as micromark's `closeFlow` does, and the containers
    // that did not continue, before it opens: an attempt still pending (a GFM table head)
    // must not see the new container's line
    exit_containers(tokenizer, &Phase::Prefix);
    // micromark closes the flow when a container opens, so what was open before cannot
    // keep the new container's first construct from interrupting it
    if let Some(child) = tokenizer.tokenize_state.document_child.as_mut() {
        child.pierce = true;
        child.interrupt = false;
    }
    tokenizer
        .tokenize_state
        .document_container_stack
        .push(container);
    tokenizer.tokenize_state.document_continued += 1;
    tokenizer.tokenize_state.document_opened = true;
    tokenizer.tokenize_state.def_list.line_boundary = true;
    State::Retry(container_new_before)
}

fn push_container(tokenizer: &mut Tokenizer, kind: Container) {
    // A failed attempt swaps the stack back, so the cache of blank ends stays valid
    let state = &mut tokenizer.tokenize_state;
    let tail = state.document_container_stack.len();
    state.document_container_stack.push(new_container(kind));
    state
        .document_container_stack
        .swap(state.document_continued, tail);
}

// The first container at or after `from` that an empty line does not continue for free
fn find_blank_end(state: &mut TokenizeState, from: usize) -> usize {
    let stack = &state.document_container_stack;
    let ends = state.document_blank_ends.get_or_insert_with(|| {
        let mut ends = vec![stack.len(); stack.len() + 1];
        for index in (0..stack.len()).rev() {
            let container = &stack[index];
            let free = match container.kind {
                Container::BlockQuote | Container::DefList => false,
                Container::GfmFootnoteDefinition => true,
                Container::ListItem => !container.blank_initial,
            };
            ends[index] = if free { ends[index + 1] } else { index };
        }
        ends
    });
    ends.get(from).copied().unwrap_or(from)
}

fn new_container(kind: Container) -> ContainerState {
    ContainerState {
        kind,
        blank_initial: false,
        size: 0,
        blank_further: false,
        blank_last: false,
    }
}

fn containers_after(tokenizer: &mut Tokenizer) -> State {
    let point = tokenizer.point.clone();
    let bytes = tokenizer.parse_state.bytes;
    let mut line_start = point.index;
    while line_start > 0 && !matches!(bytes[line_start - 1], b'\n' | b'\r') {
        line_start -= 1;
    }
    if line_start < point.index {
        let mut prefixes = tokenizer.parse_state.prefixes.borrow_mut();
        if prefixes.last().is_none_or(|last| last.0 < line_start) {
            prefixes.push((line_start, point.index));
        } else if tokenizer.parse_state.directive {
            let mut nested = tokenizer.parse_state.prefixes_nested.borrow_mut();
            nested.push((line_start, point.index));
        }
    }
    let lazy = tokenizer.tokenize_state.document_continued
        != tokenizer.tokenize_state.document_container_stack.len();
    track_def_list(tokenizer, lazy);
    if let Some(child) = tokenizer.tokenize_state.document_child.as_mut() {
        child.lazy = lazy;
        child.define_skip(point);
    }
    if tokenizer.current.is_none() {
        return State::Retry(flow_end);
    }
    let current = tokenizer.events.len();
    let previous = tokenizer.tokenize_state.document_data_index;
    if let Some(previous) = previous
        && let Some(link) = tokenizer.events[previous].link.as_mut()
    {
        link.next = Link::at(current);
    }
    tokenizer.tokenize_state.document_data_index = Some(current);
    tokenizer.enter_link(
        Name::DATA,
        Link {
            previous: previous.and_then(Link::at),
            next: None,
            content: ContentType::Flow,
        },
    );
    State::Retry(flow_inside)
}

fn flow_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.exit(Name::DATA);
            State::Retry(flow_end)
        }
        Some(b'\n') => {
            tokenizer.consume();
            tokenizer.exit(Name::DATA);
            State::Next(flow_end)
        }
        Some(_) => {
            tokenizer.consume_while(|_, _| true);
            State::Next(flow_inside)
        }
    }
}

fn flow_end(tokenizer: &mut Tokenizer) -> State {
    // The line ends with its flow chunk: in a container directive's content, the next
    // chunk starts past that line's prefix, which is not this line's flow
    let to = match tokenizer.events.last() {
        Some(event) if event.kind == Kind::Exit && event.name == Name::DATA => {
            (event.point.index, event.point.vs)
        }
        _ => (tokenizer.point.index, tokenizer.point.vs),
    };
    let state = tokenizer
        .tokenize_state
        .document_child_state
        .take()
        .unwrap_or(State::Next(tokenize_flow));
    tokenizer.tokenize_state.document_exits.push(None);
    // After a final line ending, the empty last line continues only the containers that
    // take blank lines: block quotes do not, so it is lazy inside one this line continued
    let empty_last_line_lazy =
        (tokenizer.current.is_none() && tokenizer.previous == Some(b'\n')).then(|| {
            tokenizer
                .tokenize_state
                .document_container_stack
                .iter()
                .take(tokenizer.tokenize_state.document_continued)
                .any(|container| match container.kind {
                    Container::BlockQuote => true,
                    Container::ListItem => container.blank_initial,
                    Container::DefList | Container::GfmFootnoteDefinition => false,
                })
        });
    let Some(child) = tokenizer.tokenize_state.document_child.as_mut() else {
        return State::Nok;
    };
    let from = (child.point.index, child.point.vs);
    // A blank last line, with no line ending, inside a block quote closes the containers
    // it did not continue at the end, as micromark does
    let blank_last_line = tokenizer.current.is_none()
        && tokenizer
            .tokenize_state
            .document_continued
            .checked_sub(1)
            .and_then(|last| tokenizer.tokenize_state.document_container_stack.get(last))
            .is_some_and(|container| container.kind == Container::BlockQuote)
        && tokenizer.previous != Some(b'\n')
        && tokenizer.parse_state.bytes[from.0.min(to.0)..to.0]
            .iter()
            .all(|byte| matches!(byte, b' ' | b'\t'));
    let mut state = child.push(from, to, state);
    // At the end, the last line is finished before laziness is judged, as micromark's last
    // write does: a construct still trying it (`~` as a fence) would end the paragraph
    if tokenizer.current.is_none() {
        // The flush reads the empty line's laziness; the line itself keeps its own after
        let line_lazy = child.lazy;
        if let Some(lazy) = empty_last_line_lazy {
            child.lazy = lazy;
        }
        child.flush(state, false);
        child.lazy = line_lazy;
        state = State::Ok;
    }
    tokenizer.tokenize_state.document_child_state = Some(state);

    // A paragraph (or table head) still open can take a lazy line
    let mut lazy_continuation = child
        .stack
        .iter()
        .any(|name| *name == Name::CONTENT || *name == Name::GFM_TABLE_HEAD);
    if !lazy_continuation && !child.events.is_empty() {
        let before =
            skip_opt_back(&child.events, child.events.len() - 1, &[Name::LINE_ENDING]);
        let name = child.events[before].name;
        if name == Name::CONTENT {
            lazy_continuation = true;
        }
    }
    child.pierce = false;
    let child_lazy = child.lazy;
    if child_lazy
        && tokenizer.tokenize_state.document_lazy_accepting_before
        && lazy_continuation
    {
        tokenizer.tokenize_state.document_continued =
            tokenizer.tokenize_state.document_container_stack.len();
    }
    if tokenizer.tokenize_state.document_continued
        != tokenizer.tokenize_state.document_container_stack.len()
        && !blank_last_line
    {
        exit_containers(tokenizer, &Phase::After);
    }
    if tokenizer.current.is_none() {
        tokenizer.tokenize_state.document_continued = 0;
        tokenizer.tokenize_state.document_opened = false;
        exit_containers(tokenizer, &Phase::Eof);
        resolve(tokenizer);
        State::Ok
    } else {
        tokenizer.tokenize_state.document_continued = 0;
        tokenizer.tokenize_state.document_opened = false;
        tokenizer.tokenize_state.document_lazy_accepting_before = lazy_continuation;
        tokenizer.interrupt = false;
        State::Retry(container_existing_before)
    }
}

fn exit_containers(tokenizer: &mut Tokenizer, phase: &Phase) {
    let continued = tokenizer.tokenize_state.document_continued;
    let mut stack_close = tokenizer
        .tokenize_state
        .document_container_stack
        .split_off(continued);
    tokenizer.tokenize_state.document_blank_ends = None;
    if !stack_close.is_empty() {
        let track = &mut tokenizer.tokenize_state.def_list;
        // Exits after a line come before it: it starts the runs that terms come from
        if *phase == Phase::After {
            track.run_start = track.line_start;
            track.pure_start = track.line_start;
        } else {
            track.line_boundary = true;
            track.line_exits = true;
        }
    }
    if *phase != Phase::After {
        let state = tokenizer
            .tokenize_state
            .document_child_state
            .take()
            .unwrap_or(State::Next(tokenize_flow));
        if let Some(child) = tokenizer.tokenize_state.document_child.as_mut() {
            child.flush(state, false);
        }
    }
    if !stack_close.is_empty() {
        let offset = if *phase == Phase::After { 2 } else { 1 };
        let index = tokenizer.tokenize_state.document_exits.len() - offset;
        let mut exits = Vec::with_capacity(stack_close.len());
        while let Some(container) = stack_close.pop() {
            let name = match container.kind {
                Container::BlockQuote => Name::BLOCK_QUOTE,
                Container::DefList => Name::DEF_LIST_DESCRIPTION,
                Container::GfmFootnoteDefinition => Name::GFM_FOOTNOTE_DEFINITION,
                Container::ListItem => Name::LIST_ITEM,
            };
            exits.push(Event {
                kind: Kind::Exit,
                name,
                point: tokenizer.point.clone(),
                link: None,
            });
            if let Some(position) = tokenizer.stack.iter().rposition(|open| *open == name)
            {
                tokenizer.stack.remove(position);
            }
        }
        tokenizer.tokenize_state.document_exits[index] = Some(exits);
    }
    // After a line, its own construct is still open below the closed containers: a
    // paragraph there must stay uninterruptible by indented code on the next line
    if *phase != Phase::After
        && let Some(child) = tokenizer.tokenize_state.document_child.as_mut()
    {
        child.interrupt = false;
    }
}

// Inject the container exits after the line endings they belong to, then splice the flow
// events into the document's flow chunks
fn resolve(tokenizer: &mut Tokenizer) {
    let Some(mut child) = tokenizer.tokenize_state.document_child.take() else {
        return;
    };
    let mut child_index = 0;
    let mut line = 0;
    while child_index < child.events.len() {
        let event = &child.events[child_index];
        if event.kind == Kind::Exit
            && (event.name == Name::LINE_ENDING || event.name == Name::BLANK_LINE_ENDING)
        {
            let mut inject_index = child_index - 1;
            let mut point = child.events[inject_index].point.clone();
            while child_index + 1 < child.events.len()
                && child.events[child_index + 1].kind == Kind::Exit
            {
                child_index += 1;
                point = child.events[child_index].point.clone();
                inject_index = child_index + 1;
            }
            if let Some(Some(mut exits)) = tokenizer
                .tokenize_state
                .document_exits
                .get_mut(line)
                .map(Option::take)
            {
                for exit in &mut exits {
                    exit.point = point.clone();
                }
                child.map.add(inject_index, 0, exits);
            }
            line += 1;
        } else if is_chunk_line(tokenizer.parse_state.bytes, event) {
            let mut point = event.point.clone();
            while child_index + 1 < child.events.len()
                && child.events[child_index + 1].kind == Kind::Exit
            {
                child_index += 1;
                point = child.events[child_index].point.clone();
            }
            if let Some(Some(mut exits)) = tokenizer
                .tokenize_state
                .document_exits
                .get_mut(line)
                .map(Option::take)
            {
                for exit in &mut exits {
                    exit.point = point.clone();
                }
                child.map.add(child_index + 1, 0, exits);
            }
            line += 1;
        }
        child_index += 1;
    }
    child
        .map
        .consume(&mut child.events, &child.parse_state.spare_events);

    let mut flow_index = skip_to(&tokenizer.events, 0, &[Name::DATA]);
    while flow_index < tokenizer.events.len()
        && tokenizer.events[flow_index]
            .link
            .as_ref()
            .is_none_or(|link| link.content != ContentType::Flow)
    {
        flow_index = skip_to(&tokenizer.events, flow_index + 1, &[Name::DATA]);
    }
    divide_events(
        &mut tokenizer.map,
        &tokenizer.events,
        flow_index,
        &mut child.events,
        (0, 0),
    );
    tokenizer
        .map
        .consume(&mut tokenizer.events, &tokenizer.parse_state.spare_events);

    if let Some(Some(mut exits)) = tokenizer
        .tokenize_state
        .document_exits
        .get_mut(line)
        .map(Option::take)
    {
        for exit in &mut exits {
            exit.point = tokenizer.point.clone();
        }
        tokenizer.events.append(&mut exits);
    }
    tokenizer
        .resolvers
        .append(&mut child.resolvers.split_off(0));
    // Terms are paragraph lines, so they are found once content is tokenized
    if !tokenizer.tokenize_state.def_list.starts.is_empty() {
        tokenizer.register_resolver(Resolver::DefList);
    }
    tokenizer
        .tokenize_state
        .definitions
        .append(&mut child.tokenize_state.definitions.split_off(0));
}

// A container directive's content line holds its line ending, as no token of its own
fn is_chunk_line(bytes: &[u8], event: &Event) -> bool {
    event.kind == Kind::Exit
        && event.name == CHUNK_DOCUMENT
        && event.point.index > 0
        && matches!(bytes.get(event.point.index - 1), Some(b'\n' | b'\r'))
}
