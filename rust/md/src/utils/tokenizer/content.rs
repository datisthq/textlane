use crate::actions::definition::tokenize::tokenize_definition;
use crate::actions::paragraph::tokenize::tokenize_paragraph;
use crate::models::construct::{Resolver, State};
use crate::models::event::{ContentType, Kind, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;
use crate::utils::tokenizer::subtokenize::{Subresult, subtokenize};

/// One line of content: definitions and paragraphs, tokenized once the lines are joined.
pub(crate) fn tokenize_content_chunk(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => State::Nok,
        _ => {
            if tokenizer.tokenize_state.content_break_line.take()
                == Some(tokenizer.point.line as usize)
            {
                let start = tokenizer.point.index;
                tokenizer
                    .parse_state
                    .content_breaks
                    .borrow_mut()
                    .insert(start as usize);
            }
            tokenizer.enter_link(
                Name::CONTENT,
                Link {
                    previous: None,
                    next: None,
                    content: ContentType::Content,
                },
            );
            State::Retry(chunk_inside)
        }
    }
}

fn chunk_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::CONTENT);
            tokenizer.register_resolver_before(Resolver::Content);
            // Content is open: a following line may continue it or interrupt it
            tokenizer.interrupt = true;
            State::Ok
        }
        _ => {
            tokenizer.consume_while(|_, _| true);
            State::Next(chunk_inside)
        }
    }
}

/// The content content type: definitions first, then a paragraph.
pub(crate) fn tokenize_content_definition_before(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(
        State::Next(definition_after),
        State::Next(tokenize_paragraph),
    );
    State::Retry(tokenize_definition)
}

fn definition_after(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current.is_none() {
        return State::Ok;
    }
    tokenizer.enter(Name::LINE_ENDING);
    tokenizer.consume();
    tokenizer.exit(Name::LINE_ENDING);
    State::Next(tokenize_content_definition_before)
}

/// Join adjacent content chunks into one chain, then tokenize the chains.
pub(crate) fn resolve_content(tokenizer: &mut Tokenizer) -> Subresult {
    let mut index = 0;
    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];
        if event.kind == Kind::Enter && event.name == Name::CONTENT {
            let mut exit_index = index + 1;
            loop {
                let mut enter_index = exit_index + 1;
                if enter_index == tokenizer.events.len()
                    || tokenizer.events[enter_index].name != Name::LINE_ENDING
                {
                    break;
                }
                enter_index += 2;
                while enter_index < tokenizer.events.len() {
                    let name = tokenizer.events[enter_index].name;
                    if name != Name::SPACE_OR_TAB
                        && name != Name::BLOCK_QUOTE_PREFIX
                        && name != Name::BLOCK_QUOTE_MARKER
                    {
                        break;
                    }
                    enter_index += 1;
                }
                if enter_index == tokenizer.events.len()
                    || tokenizer.events[enter_index].name != Name::CONTENT
                    || tokenizer
                        .parse_state
                        .content_breaks
                        .borrow()
                        .contains(&(tokenizer.events[enter_index].point.index as usize))
                {
                    break;
                }
                // Swallow the line ending into the chunk before it
                tokenizer.events[exit_index].point =
                    tokenizer.events[exit_index + 2].point;
                tokenizer.map.add(exit_index + 1, 2, vec![]);
                if let Some(link) = tokenizer.events[exit_index - 1].link.as_mut() {
                    link.next = Link::at(enter_index);
                }
                if let Some(link) = tokenizer.events[enter_index].link.as_mut() {
                    link.previous = Link::at(exit_index - 1);
                }
                exit_index = enter_index + 1;
            }
            index = exit_index;
        }
        index += 1;
    }
    tokenizer.map.consume(&mut tokenizer.events);
    subtokenize(
        &mut tokenizer.events,
        tokenizer.parse_state,
        Some(ContentType::Content),
    )
}
