use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab;
use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const TAB_SIZE: usize = 4;
const MARKER_COUNT_MIN: usize = 3;

/// A thematic break: three or more of one marker, `*`, `-` or `_`.
pub(crate) fn tokenize_thematic_break(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::THEMATIC_BREAK);
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(before), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 0, TAB_SIZE - 1))
    } else {
        State::Retry(before)
    }
}

fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(marker @ (b'*' | b'-' | b'_')) => {
            // A break failed from earlier on this line fails from here too: only markers
            // and spaces lie between, and checking again for each nested list item would
            // make input quadratic
            let index = tokenizer.point.index;
            if let Some((failed, start, end)) =
                tokenizer.tokenize_state.thematic_break_failed
                && failed == marker
                && (start..end).contains(&index)
            {
                return State::Nok;
            }
            tokenizer.tokenize_state.marker = marker;
            tokenizer.tokenize_state.size_c = index;
            State::Retry(at_break)
        }
        _ => State::Nok,
    }
}

fn at_break(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(tokenizer.tokenize_state.marker) {
        tokenizer.enter(Name::THEMATIC_BREAK_SEQUENCE);
        State::Retry(sequence)
    } else if tokenizer.tokenize_state.size >= MARKER_COUNT_MIN
        && matches!(tokenizer.current, None | Some(b'\n'))
    {
        tokenizer.tokenize_state.marker = 0;
        tokenizer.tokenize_state.size = 0;
        tokenizer.exit(Name::THEMATIC_BREAK);
        tokenizer.interrupt = false;
        State::Ok
    } else {
        let state = &mut tokenizer.tokenize_state;
        state.thematic_break_failed =
            Some((state.marker, state.size_c, tokenizer.point.index));
        state.marker = 0;
        state.size = 0;
        State::Nok
    }
}

fn sequence(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(tokenizer.tokenize_state.marker) {
        tokenizer.consume();
        tokenizer.tokenize_state.size += 1;
        State::Next(sequence)
    } else if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.exit(Name::THEMATIC_BREAK_SEQUENCE);
        tokenizer.attempt(State::Next(at_break), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
    } else {
        tokenizer.exit(Name::THEMATIC_BREAK_SEQUENCE);
        State::Retry(at_break)
    }
}
