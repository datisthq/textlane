use crate::actions::directive_attributes::tokenize::tokenize_directive_attributes;
use crate::actions::directive_label::tokenize::tokenize_directive_label;
use crate::actions::directive_name::tokenize::tokenize_directive_name;
use crate::actions::non_lazy_continuation::tokenize::tokenize_non_lazy_continuation;
use crate::actions::space_or_tab::tokenize::{
    tokenize_space_or_tab, tokenize_space_or_tab_indent,
};
use crate::helpers::slice::{Slice, span_of_exit};
use crate::models::construct::{Resolver, State};
use crate::models::directive::{
    CHUNK_DOCUMENT, DIRECTIVE_CONTAINER, DIRECTIVE_CONTAINER_CONTENT,
    DIRECTIVE_CONTAINER_FENCE, DIRECTIVE_CONTAINER_SEQUENCE, DirectiveKind,
};
use crate::models::event::{ContentType, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const SEQUENCE_SIZE_MIN: usize = 3;

/// A container directive: an opening fence of three or more colons with a name, label and
/// attributes, content lines tokenized as a document of their own, and a closing fence
/// at least as long.
pub(crate) fn tokenize_directive_container(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(before), State::Nok);
        return State::Retry(tokenize_space_or_tab_indent(tokenizer, 0));
    }
    State::Retry(before)
}

fn before(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b':') {
        return State::Nok;
    }
    // The opening fence's indent is removed from each content line
    let mut prefix = 0;
    if let Some(event) = tokenizer.events.last()
        && event.name == Name::SPACE_OR_TAB
    {
        let (start, end) = span_of_exit(&tokenizer.events, tokenizer.events.len() - 1);
        prefix = Slice::from_points(tokenizer.parse_state.bytes, start, end).len();
    }
    let state = &mut tokenizer.tokenize_state.directive;
    state.kind = DirectiveKind::Container;
    state.size = 0;
    state.prefix = prefix;
    tokenizer.enter(DIRECTIVE_CONTAINER);
    tokenizer.enter(DIRECTIVE_CONTAINER_FENCE);
    tokenizer.enter(DIRECTIVE_CONTAINER_SEQUENCE);
    State::Retry(sequence_open)
}

fn sequence_open(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b':') {
        tokenizer.consume();
        tokenizer.tokenize_state.directive.size += 1;
        return State::Next(sequence_open);
    }
    if tokenizer.tokenize_state.directive.size < SEQUENCE_SIZE_MIN {
        return State::Nok;
    }
    tokenizer.exit(DIRECTIVE_CONTAINER_SEQUENCE);
    tokenizer.attempt(State::Next(after_name), State::Nok);
    State::Retry(tokenize_directive_name)
}

fn after_name(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'[') {
        tokenizer.attempt(State::Next(after_label), State::Next(after_label));
        return State::Retry(tokenize_directive_label);
    }
    State::Retry(after_label)
}

fn after_label(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'{') {
        tokenizer.attempt(State::Next(after_attributes), State::Next(after_attributes));
        return State::Retry(tokenize_directive_attributes);
    }
    State::Retry(after_attributes)
}

fn after_attributes(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(open_after), State::Nok);
        return State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX));
    }
    State::Retry(open_after)
}

fn open_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.exit(DIRECTIVE_CONTAINER_FENCE);
            State::Retry(after)
        }
        Some(b'\n') => {
            tokenizer.exit(DIRECTIVE_CONTAINER_FENCE);
            // Like a fence, the container is a whole construct: containers cannot open
            // inside it, and lazy lines end it
            tokenizer.concrete = true;
            tokenizer.attempt(State::Next(content_start), State::Next(after));
            State::Retry(tokenize_non_lazy_continuation)
        }
        Some(_) => State::Nok,
    }
}

fn content_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Retry(after),
        Some(b'\n') => {
            tokenizer.check(State::Next(content_enter), State::Next(after));
            State::Retry(tokenize_non_lazy_continuation)
        }
        Some(_) => State::Retry(content_enter),
    }
}

fn content_enter(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(DIRECTIVE_CONTAINER_CONTENT);
    State::Retry(line_start)
}

fn line_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after_content), State::Next(line_prefix));
    State::Retry(closing_start)
}

fn line_prefix(tokenizer: &mut Tokenizer) -> State {
    let prefix = tokenizer.tokenize_state.directive.prefix;
    if prefix > 0 && matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(chunk_start), State::Nok);
        return State::Retry(tokenize_space_or_tab(tokenizer, 0, prefix));
    }
    State::Retry(chunk_start)
}

fn chunk_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Retry(after_content),
        Some(b'\n') => {
            tokenizer.check(State::Next(chunk_enter), State::Next(after_content));
            State::Retry(tokenize_non_lazy_continuation)
        }
        Some(_) => State::Retry(chunk_enter),
    }
}

// Chunks are linked once the document has its events in place: lines of a container
// inside a list or block quote are divided over the document's flow chunks first
fn chunk_enter(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter_link(
        CHUNK_DOCUMENT,
        Link {
            previous: None,
            next: None,
            content: ContentType::Document,
        },
    );
    tokenizer.register_resolver(Resolver::DirectiveContainer);
    State::Retry(chunk_inside)
}

fn chunk_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.exit(CHUNK_DOCUMENT);
            State::Retry(after_content)
        }
        Some(b'\n') => {
            tokenizer.check(State::Next(chunk_line_ending), State::Next(chunk_exit));
            State::Retry(tokenize_non_lazy_continuation)
        }
        Some(_) => {
            tokenizer.consume();
            State::Next(chunk_inside)
        }
    }
}

fn chunk_line_ending(tokenizer: &mut Tokenizer) -> State {
    tokenizer.consume();
    tokenizer.exit(CHUNK_DOCUMENT);
    State::Next(line_start)
}

fn chunk_exit(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(CHUNK_DOCUMENT);
    State::Retry(after_content)
}

fn after_content(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(DIRECTIVE_CONTAINER_CONTENT);
    State::Retry(after)
}

fn after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(DIRECTIVE_CONTAINER);
    tokenizer.concrete = false;
    tokenizer.interrupt = false;
    State::Ok
}

fn closing_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.directive.size_close = 0;
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(closing_prefix_after), State::Nok);
        return State::Retry(tokenize_space_or_tab_indent(tokenizer, 0));
    }
    State::Retry(closing_prefix_after)
}

fn closing_prefix_after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(DIRECTIVE_CONTAINER_FENCE);
    tokenizer.enter(DIRECTIVE_CONTAINER_SEQUENCE);
    State::Retry(closing_sequence)
}

fn closing_sequence(tokenizer: &mut Tokenizer) -> State {
    let state = &mut tokenizer.tokenize_state.directive;
    if tokenizer.current == Some(b':') {
        state.size_close += 1;
        tokenizer.consume();
        return State::Next(closing_sequence);
    }
    if state.size_close < state.size {
        return State::Nok;
    }
    tokenizer.exit(DIRECTIVE_CONTAINER_SEQUENCE);
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(closing_end), State::Nok);
        return State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX));
    }
    State::Retry(closing_end)
}

fn closing_end(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(DIRECTIVE_CONTAINER_FENCE);
            State::Ok
        }
        _ => State::Nok,
    }
}
