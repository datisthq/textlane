use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab;
use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const TAB_SIZE: usize = 4;

/// Indented code: lines indented by four spaces or more, which cannot interrupt a paragraph.
pub(crate) fn tokenize_code_indented(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.interrupt
        || !tokenizer.parse_state.code_indented()
        || !matches!(tokenizer.current, Some(b'\t' | b' '))
    {
        return State::Nok;
    }
    tokenizer.enter(Name::CODE_INDENTED);
    tokenizer.attempt(State::Next(at_break), State::Nok);
    State::Retry(tokenize_space_or_tab(tokenizer, TAB_SIZE, TAB_SIZE))
}

fn at_break(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Retry(after),
        Some(b'\n') => {
            // Open code is a construct in progress: micromark counts any such as interrupting,
            // so an empty list item cannot start on the next line
            tokenizer.interrupt = true;
            tokenizer.attempt(State::Next(at_break), State::Next(after));
            State::Retry(further_line_ending)
        }
        _ => {
            tokenizer.enter(Name::CODE_FLOW_CHUNK);
            State::Retry(inside)
        }
    }
}

fn inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::CODE_FLOW_CHUNK);
            State::Retry(at_break)
        }
        _ => {
            tokenizer.consume();
            State::Next(inside)
        }
    }
}

fn after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(Name::CODE_INDENTED);
    tokenizer.interrupt = false;
    State::Ok
}

// The code's own line ending: micromark ends code on a lazy line here, but a container
// that opened on this line does not end it, as piercing would
fn further_line_ending(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.lazy {
        return State::Nok;
    }
    tokenizer.enter(Name::LINE_ENDING);
    tokenizer.consume();
    tokenizer.exit(Name::LINE_ENDING);
    State::Next(further_start)
}

fn further_start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.lazy || tokenizer.pierce {
        return State::Nok;
    }
    if tokenizer.current == Some(b'\n') {
        tokenizer.enter(Name::LINE_ENDING);
        tokenizer.consume();
        tokenizer.exit(Name::LINE_ENDING);
        State::Next(further_start)
    } else {
        tokenizer.attempt(State::Ok, State::Next(further_begin));
        State::Retry(tokenize_space_or_tab(tokenizer, TAB_SIZE, TAB_SIZE))
    }
}

// A blank line inside the code, if more indented code follows it
fn further_begin(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(further_after), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
    } else {
        State::Nok
    }
}

fn further_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => State::Retry(further_start),
        _ => State::Nok,
    }
}
