use crate::models::construct::State;
use crate::models::event::{ContentType, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;
use crate::utils::tokenizer::subtokenize::link;

/// A paragraph: the rest of the content, each line a chunk of linked text.
pub(crate) fn tokenize_paragraph(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::PARAGRAPH);
    State::Retry(line_start)
}

fn line_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter_link(
        Name::DATA,
        Link {
            previous: None,
            next: None,
            content: ContentType::Text,
        },
    );
    if tokenizer.tokenize_state.connect {
        let index = tokenizer.events.len() - 1;
        link(&mut tokenizer.events, index);
    } else {
        tokenizer.tokenize_state.connect = true;
    }
    State::Retry(inside)
}

fn inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.tokenize_state.connect = false;
            tokenizer.exit(Name::DATA);
            tokenizer.exit(Name::PARAGRAPH);
            State::Ok
        }
        Some(b'\n') => {
            tokenizer.consume();
            tokenizer.exit(Name::DATA);
            State::Next(line_start)
        }
        _ => {
            tokenizer.consume_while(|_, _| true);
            State::Next(inside)
        }
    }
}
