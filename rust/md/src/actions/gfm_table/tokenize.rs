use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab;
use crate::helpers::skip::skip_opt_back;
use crate::models::construct::{Resolver, State};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const TAB_SIZE: usize = 4;

/// A GFM table: a head row and a delimiter row, then a body row per line.
pub(crate) fn tokenize_gfm_table(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.gfm.is_none() {
        return State::Nok;
    }
    let after_row = !tokenizer.pierce
        && !tokenizer.events.is_empty()
        && matches!(
            tokenizer.events[skip_opt_back(
                &tokenizer.events,
                tokenizer.events.len() - 1,
                &[Name::LINE_ENDING, Name::SPACE_OR_TAB],
            )]
            .name,
            Name::GFM_TABLE_HEAD | Name::GFM_TABLE_ROW
        );
    if after_row {
        State::Retry(body_row_before)
    } else {
        State::Retry(head_row_before)
    }
}

// The indent goes before the row: micromark eats it as a line prefix, so the table, its
// rows and their first cells start after it
fn head_row_before(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(head_row_open), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 0, TAB_SIZE - 1))
    } else {
        State::Retry(head_row_open)
    }
}

fn head_row_open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        // Four or more spaces are indented code
        Some(b'\t' | b' ') => State::Nok,
        _ => {
            tokenizer.enter(Name::GFM_TABLE_HEAD);
            tokenizer.enter(Name::GFM_TABLE_ROW);
            State::Retry(head_row_start)
        }
    }
}

fn head_row_start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'|') {
        tokenizer.tokenize_state.seen = true;
        // The first character that is not a pipe counts double
        tokenizer.tokenize_state.size_b += 1;
    }
    State::Retry(head_row_break)
}

fn head_row_break(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Retry(head_nok),
        Some(b'\n') => {
            // Anything but one pipe, whitespace aside, makes a head row
            if tokenizer.tokenize_state.size_b > 1 {
                tokenizer.tokenize_state.size_b = 0;
                tokenizer.interrupt = true;
                tokenizer.exit(Name::GFM_TABLE_ROW);
                tokenizer.enter(Name::LINE_ENDING);
                tokenizer.consume();
                tokenizer.exit(Name::LINE_ENDING);
                State::Next(head_delimiter_start)
            } else {
                State::Retry(head_nok)
            }
        }
        Some(b'\t' | b' ') => {
            tokenizer.attempt(State::Next(head_row_break), State::Nok);
            State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
        }
        Some(byte) => {
            tokenizer.tokenize_state.size_b += 1;
            if tokenizer.tokenize_state.seen {
                tokenizer.tokenize_state.seen = false;
                tokenizer.tokenize_state.size += 1;
            }
            if byte == b'|' {
                tokenizer.enter(Name::GFM_TABLE_CELL_DIVIDER);
                tokenizer.consume();
                tokenizer.exit(Name::GFM_TABLE_CELL_DIVIDER);
                tokenizer.tokenize_state.seen = true;
                State::Next(head_row_break)
            } else {
                tokenizer.enter(Name::DATA);
                State::Retry(head_row_data)
            }
        }
    }
}

fn head_row_data(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\t' | b'\n' | b' ' | b'|') => {
            tokenizer.exit(Name::DATA);
            State::Retry(head_row_break)
        }
        Some(byte) => {
            tokenizer.consume();
            if byte == b'\\' {
                State::Next(head_row_escape)
            } else {
                State::Next(head_row_data)
            }
        }
    }
}

fn head_row_escape(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\\' | b'|') => {
            tokenizer.consume();
            State::Next(head_row_data)
        }
        _ => State::Retry(head_row_data),
    }
}

fn head_delimiter_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.interrupt = false;
    if tokenizer.lazy || tokenizer.pierce {
        return State::Retry(head_nok);
    }
    tokenizer.enter(Name::GFM_TABLE_DELIMITER_ROW);
    tokenizer.tokenize_state.seen = false;
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(head_delimiter_before), State::Next(head_nok));
        State::Retry(tokenize_space_or_tab(tokenizer, 0, TAB_SIZE - 1))
    } else {
        State::Retry(head_delimiter_before)
    }
}

fn head_delimiter_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-' | b':') => State::Retry(head_delimiter_value_before),
        Some(b'|') => {
            tokenizer.tokenize_state.seen = true;
            tokenizer.enter(Name::GFM_TABLE_CELL_DIVIDER);
            tokenizer.consume();
            tokenizer.exit(Name::GFM_TABLE_CELL_DIVIDER);
            State::Next(head_delimiter_cell_before)
        }
        _ => State::Retry(head_nok),
    }
}

fn head_delimiter_cell_before(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(head_delimiter_value_before), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
    } else {
        State::Retry(head_delimiter_value_before)
    }
}

fn head_delimiter_value_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => State::Retry(head_delimiter_cell_after),
        Some(b':') => {
            tokenizer.tokenize_state.size_b += 1;
            tokenizer.tokenize_state.seen = true;
            tokenizer.enter(Name::GFM_TABLE_DELIMITER_MARKER);
            tokenizer.consume();
            tokenizer.exit(Name::GFM_TABLE_DELIMITER_MARKER);
            State::Next(head_delimiter_left_alignment_after)
        }
        Some(b'-') => {
            tokenizer.tokenize_state.size_b += 1;
            State::Retry(head_delimiter_left_alignment_after)
        }
        _ => State::Retry(head_nok),
    }
}

fn head_delimiter_left_alignment_after(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'-') {
        tokenizer.enter(Name::GFM_TABLE_DELIMITER_FILLER);
        State::Retry(head_delimiter_filler)
    } else {
        State::Retry(head_nok)
    }
}

fn head_delimiter_filler(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-') => {
            tokenizer.consume();
            State::Next(head_delimiter_filler)
        }
        Some(b':') => {
            tokenizer.tokenize_state.seen = true;
            tokenizer.exit(Name::GFM_TABLE_DELIMITER_FILLER);
            tokenizer.enter(Name::GFM_TABLE_DELIMITER_MARKER);
            tokenizer.consume();
            tokenizer.exit(Name::GFM_TABLE_DELIMITER_MARKER);
            State::Next(head_delimiter_right_alignment_after)
        }
        _ => {
            tokenizer.exit(Name::GFM_TABLE_DELIMITER_FILLER);
            State::Retry(head_delimiter_right_alignment_after)
        }
    }
}

fn head_delimiter_right_alignment_after(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(head_delimiter_cell_after), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
    } else {
        State::Retry(head_delimiter_cell_after)
    }
}

fn head_delimiter_cell_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            // No `:` or `|` at all is a thematic break or setext underline instead
            if !tokenizer.tokenize_state.seen
                || tokenizer.tokenize_state.size != tokenizer.tokenize_state.size_b
            {
                return State::Retry(head_nok);
            }
            reset(tokenizer);
            tokenizer.exit(Name::GFM_TABLE_DELIMITER_ROW);
            tokenizer.exit(Name::GFM_TABLE_HEAD);
            tokenizer.register_resolver(Resolver::GfmTable);
            State::Ok
        }
        Some(b'|') => State::Retry(head_delimiter_before),
        _ => State::Retry(head_nok),
    }
}

fn head_nok(tokenizer: &mut Tokenizer) -> State {
    reset(tokenizer);
    State::Nok
}

fn reset(tokenizer: &mut Tokenizer) {
    tokenizer.tokenize_state.seen = false;
    tokenizer.tokenize_state.size = 0;
    tokenizer.tokenize_state.size_b = 0;
}

fn body_row_before(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.lazy {
        return State::Nok;
    }
    // Blank lines and indented code were tried before, so any indent is fine
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(body_row_open), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 0, usize::MAX))
    } else {
        State::Retry(body_row_open)
    }
}

fn body_row_open(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::GFM_TABLE_ROW);
    State::Retry(body_row_break)
}

fn body_row_break(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::GFM_TABLE_ROW);
            State::Ok
        }
        Some(b'\t' | b' ') => {
            tokenizer.attempt(State::Next(body_row_break), State::Nok);
            State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
        }
        Some(b'|') => {
            tokenizer.enter(Name::GFM_TABLE_CELL_DIVIDER);
            tokenizer.consume();
            tokenizer.exit(Name::GFM_TABLE_CELL_DIVIDER);
            State::Next(body_row_break)
        }
        Some(_) => {
            tokenizer.enter(Name::DATA);
            State::Retry(body_row_data)
        }
    }
}

fn body_row_data(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\t' | b'\n' | b' ' | b'|') => {
            tokenizer.exit(Name::DATA);
            State::Retry(body_row_break)
        }
        Some(byte) => {
            tokenizer.consume();
            if byte == b'\\' {
                State::Next(body_row_escape)
            } else {
                State::Next(body_row_data)
            }
        }
    }
}

fn body_row_escape(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\\' | b'|') => {
            tokenizer.consume();
            State::Next(body_row_data)
        }
        _ => State::Retry(body_row_data),
    }
}
