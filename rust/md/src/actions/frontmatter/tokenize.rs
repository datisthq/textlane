use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const BOM: &[u8] = "\u{FEFF}".as_bytes();

/// Frontmatter: an opening fence on the first line, or any line for a matter found
/// `anywhere`, the value, and a closing fence; each matter whose fence starts with the
/// current byte is tried in order.
pub(crate) fn tokenize_frontmatter(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.frontmatter_index = 0;
    State::Retry(matter_before)
}

fn matter_before(tokenizer: &mut Tokenizer) -> State {
    let point = &tokenizer.point;
    let state = &tokenizer.tokenize_state;
    let matters = &tokenizer.parse_state.frontmatter;
    // micromark drops a byte order mark before it tokenizes, so the column after it is 1
    let bom = point.line == 1
        && point.index == BOM.len()
        && tokenizer.parse_state.bytes.starts_with(BOM);
    let found = (state.frontmatter_index..matters.len()).find(|&index| {
        let matter = &matters[index];
        (point.column == 1 || bom)
            && point.vs == 0
            && (point.line == 1 || matter.anywhere)
            && matter.open.as_bytes().first() == tokenizer.current.as_ref()
            && state
                .frontmatter_failed
                .get(index)
                .is_none_or(|failed| failed.is_none_or(|start| point.index < start))
    });
    let Some(index) = found else {
        return State::Nok;
    };
    tokenizer.tokenize_state.frontmatter_index = index;
    tokenizer.tokenize_state.frontmatter_start = tokenizer.point.index;
    tokenizer.attempt(State::Ok, State::Next(matter_next));
    State::Retry(open_start)
}

fn matter_next(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.frontmatter_index += 1;
    State::Retry(matter_before)
}

fn open_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::FRONTMATTER);
    tokenizer.enter(Name::FRONTMATTER_FENCE);
    tokenizer.enter(Name::FRONTMATTER_SEQUENCE);
    tokenizer.tokenize_state.frontmatter_fence = 0;
    State::Retry(open_sequence)
}

fn open_sequence(tokenizer: &mut Tokenizer) -> State {
    match sequence(tokenizer, true) {
        Some(state) => state,
        None => {
            tokenizer.exit(Name::FRONTMATTER_SEQUENCE);
            State::Retry(open_whitespace)
        }
    }
}

fn open_whitespace(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b' ') => {
            whitespace(tokenizer);
            State::Next(open_whitespace)
        }
        Some(b'\n') => {
            whitespace_end(tokenizer);
            tokenizer.exit(Name::FRONTMATTER_FENCE);
            tokenizer.enter(Name::LINE_ENDING);
            tokenizer.consume();
            tokenizer.exit(Name::LINE_ENDING);
            // Lines inside are the matter's: no container may start on them
            tokenizer.concrete = true;
            State::Next(close_before)
        }
        _ => nok(tokenizer),
    }
}

fn close_before(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(after), State::Next(content_start));
    State::Retry(close_start)
}

fn close_start(tokenizer: &mut Tokenizer) -> State {
    let close = read_fence(tokenizer, false);
    if close.is_empty() || close.first() != tokenizer.current.as_ref() {
        return State::Nok;
    }
    tokenizer.enter(Name::FRONTMATTER_FENCE);
    tokenizer.enter(Name::FRONTMATTER_SEQUENCE);
    tokenizer.tokenize_state.frontmatter_fence = 0;
    State::Retry(close_sequence)
}

fn close_sequence(tokenizer: &mut Tokenizer) -> State {
    match sequence(tokenizer, false) {
        Some(state) => state,
        None => {
            tokenizer.exit(Name::FRONTMATTER_SEQUENCE);
            State::Retry(close_whitespace)
        }
    }
}

fn close_whitespace(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b' ') => {
            whitespace(tokenizer);
            State::Next(close_whitespace)
        }
        None | Some(b'\n') => {
            whitespace_end(tokenizer);
            tokenizer.exit(Name::FRONTMATTER_FENCE);
            State::Ok
        }
        _ => State::Nok,
    }
}

fn content_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => State::Retry(content_end),
        _ => {
            tokenizer.enter(Name::FRONTMATTER_VALUE);
            State::Retry(content_inside)
        }
    }
}

fn content_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::FRONTMATTER_VALUE);
            State::Retry(content_end)
        }
        _ => {
            tokenizer.consume();
            State::Next(content_inside)
        }
    }
}

fn content_end(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => {
            tokenizer.enter(Name::LINE_ENDING);
            tokenizer.consume();
            tokenizer.exit(Name::LINE_ENDING);
            State::Next(close_before)
        }
        _ => {
            // No closing fence follows: one opening on a later line fails the same way
            let state = &mut tokenizer.tokenize_state;
            let index = state.frontmatter_index;
            if state.frontmatter_failed.len() <= index {
                state.frontmatter_failed.resize(index + 1, None);
            }
            state.frontmatter_failed[index].get_or_insert(state.frontmatter_start);
            nok(tokenizer)
        }
    }
}

fn after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(Name::FRONTMATTER);
    tokenizer.concrete = false;
    tokenizer.interrupt = false;
    State::Ok
}

fn nok(tokenizer: &mut Tokenizer) -> State {
    tokenizer.concrete = false;
    State::Nok
}

// Consume the next byte of the fence; `None` once all of it is consumed
fn sequence(tokenizer: &mut Tokenizer, open: bool) -> Option<State> {
    let fence = read_fence(tokenizer, open);
    let index = tokenizer.tokenize_state.frontmatter_fence;
    let byte = *fence.get(index)?;
    if tokenizer.current != Some(byte) {
        return Some(State::Nok);
    }
    tokenizer.tokenize_state.frontmatter_fence += 1;
    tokenizer.consume();
    Some(State::Next(if open {
        open_sequence
    } else {
        close_sequence
    }))
}

fn read_fence<'a>(tokenizer: &Tokenizer<'a>, open: bool) -> &'a [u8] {
    let matters = &tokenizer.parse_state.frontmatter;
    match matters.get(tokenizer.tokenize_state.frontmatter_index) {
        Some(matter) if open => matter.open.as_bytes(),
        Some(matter) => matter.close.as_bytes(),
        None => &[],
    }
}

fn whitespace(tokenizer: &mut Tokenizer) {
    if tokenizer.stack.last() != Some(&Name::SPACE_OR_TAB) {
        tokenizer.enter(Name::SPACE_OR_TAB);
    }
    tokenizer.consume();
}

fn whitespace_end(tokenizer: &mut Tokenizer) {
    if tokenizer.stack.last() == Some(&Name::SPACE_OR_TAB) {
        tokenizer.exit(Name::SPACE_OR_TAB);
    }
}
