use crate::actions::bom::tokenize::tokenize_bom;
use crate::helpers::skip::{skip_opt_back, skip_to};
use crate::models::construct::State;
use crate::models::event::{ContentType, Event, Kind, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::{Container, Tokenizer};
use crate::utils::tokenizer::flow::tokenize_flow;
use crate::utils::tokenizer::subtokenize::divide_events;

// Block quotes and list items exit with `Prefix` once they exist
#[allow(dead_code)]
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
    tokenizer.attempt(
        State::Next(container_existing_before),
        State::Next(container_existing_before),
    );
    State::Retry(tokenize_bom)
}

fn container_existing_before(_tokenizer: &mut Tokenizer) -> State {
    // Block quotes and list items continue here once they exist
    State::Retry(container_new_before)
}

fn container_new_before(tokenizer: &mut Tokenizer) -> State {
    let state = &tokenizer.tokenize_state;
    if state.document_continued == state.document_container_stack.len()
        && let Some(child) = state.document_child.as_ref()
    {
        tokenizer.interrupt = child.interrupt;
        if child.concrete {
            return State::Retry(containers_after);
        }
    }
    // Block quotes and list items start here once they exist
    State::Retry(containers_after)
}

fn containers_after(tokenizer: &mut Tokenizer) -> State {
    let point = tokenizer.point.clone();
    let lazy = tokenizer.tokenize_state.document_continued
        != tokenizer.tokenize_state.document_container_stack.len();
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
        link.next = Some(current);
    }
    tokenizer.tokenize_state.document_data_index = Some(current);
    tokenizer.enter_link(
        Name::DATA,
        Link {
            previous,
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
            tokenizer.consume();
            State::Next(flow_inside)
        }
    }
}

fn flow_end(tokenizer: &mut Tokenizer) -> State {
    let to = (tokenizer.point.index, tokenizer.point.vs);
    let state = tokenizer
        .tokenize_state
        .document_child_state
        .take()
        .unwrap_or(State::Next(tokenize_flow));
    tokenizer.tokenize_state.document_exits.push(None);
    let Some(child) = tokenizer.tokenize_state.document_child.as_mut() else {
        return State::Nok;
    };
    let from = (child.point.index, child.point.vs);
    let state = child.push(from, to, state);
    tokenizer.tokenize_state.document_child_state = Some(state);

    // A paragraph (or table head) still open can take a lazy line
    let mut lazy_continuation = child
        .stack
        .iter()
        .any(|name| *name == Name::CONTENT || *name == Name("gfmTableHead"));
    if !lazy_continuation && !child.events.is_empty() {
        let before =
            skip_opt_back(&child.events, child.events.len() - 1, &[Name::LINE_ENDING]);
        let name = child.events[before].name;
        if name == Name::CONTENT || name == Name::HEADING_SETEXT_UNDERLINE {
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
    {
        exit_containers(tokenizer, &Phase::After);
    }
    if tokenizer.current.is_none() {
        tokenizer.tokenize_state.document_continued = 0;
        exit_containers(tokenizer, &Phase::Eof);
        resolve(tokenizer);
        State::Ok
    } else {
        tokenizer.tokenize_state.document_continued = 0;
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
                Container::BlockQuote => Name("blockQuote"),
                Container::ListItem => Name("listItem"),
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
    if let Some(child) = tokenizer.tokenize_state.document_child.as_mut() {
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
        }
        child_index += 1;
    }
    child.map.consume(&mut child.events);

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
    tokenizer.map.consume(&mut tokenizer.events);

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
    tokenizer
        .tokenize_state
        .definitions
        .append(&mut child.tokenize_state.definitions.split_off(0));
}
