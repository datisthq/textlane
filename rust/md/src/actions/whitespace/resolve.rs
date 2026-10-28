use crate::helpers::slice::{Slice, span_of_exit};
use crate::models::event::{Event, Kind};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const HARD_BREAK_PREFIX_SIZE_MIN: usize = 2;

/// Split the spaces and tabs at the edges of each data token off into their own tokens;
/// two or more trailing spaces before a line ending become a hard break when `hard_break`.
pub(crate) fn resolve_whitespace(
    tokenizer: &mut Tokenizer,
    hard_break: bool,
    trim_whole: bool,
) {
    let len = tokenizer.events.len();
    for index in 0..len {
        let event = &tokenizer.events[index];
        if event.kind == Kind::Exit && event.name == Name::DATA {
            let trim_start = (trim_whole && index == 1)
                || (index > 1 && tokenizer.events[index - 2].name == Name::LINE_ENDING);
            let trim_end = (trim_whole && index == len - 1)
                || (index + 1 < len
                    && tokenizer.events[index + 1].name == Name::LINE_ENDING);
            trim_data(tokenizer, index, trim_start, trim_end, hard_break);
        }
    }
    tokenizer.map.consume(&mut tokenizer.events);
}

fn trim_data(
    tokenizer: &mut Tokenizer,
    exit_index: usize,
    trim_start: bool,
    trim_end: bool,
    hard_break: bool,
) {
    let (start, end) = span_of_exit(&tokenizer.events, exit_index);
    let mut slice = Slice::from_points(tokenizer.parse_state.bytes, start, end);

    if trim_end {
        let mut index = slice.bytes.len();
        let mut spaces_only = slice.after == 0;
        while index > 0 {
            match slice.bytes[index - 1] {
                b' ' => {}
                b'\t' => spaces_only = false,
                _ => break,
            }
            index -= 1;
        }
        let diff = slice.bytes.len() - index;
        let name = if hard_break
            && spaces_only
            && diff >= HARD_BREAK_PREFIX_SIZE_MIN
            && exit_index + 1 < tokenizer.events.len()
        {
            Name::HARD_BREAK_TRAILING
        } else {
            Name::SPACE_OR_TAB
        };
        // All whitespace: the whole token becomes it
        if index == 0 {
            tokenizer.events[exit_index - 1].name = name;
            tokenizer.events[exit_index].name = name;
            return;
        }
        if diff > 0 || slice.after > 0 {
            let exit_point = tokenizer.events[exit_index].point.clone();
            let mut enter_point = exit_point.clone();
            enter_point.index -= diff;
            enter_point.column -= diff;
            enter_point.vs = 0;
            tokenizer.map.add(
                exit_index + 1,
                0,
                vec![
                    Event {
                        kind: Kind::Enter,
                        name,
                        point: enter_point.clone(),
                        link: None,
                    },
                    Event {
                        kind: Kind::Exit,
                        name,
                        point: exit_point,
                        link: None,
                    },
                ],
            );
            tokenizer.events[exit_index].point = enter_point;
            slice.bytes = &slice.bytes[..index];
        }
    }

    if trim_start {
        let index = slice
            .bytes
            .iter()
            .position(|byte| !matches!(byte, b' ' | b'\t'))
            .unwrap_or(slice.bytes.len());
        if index == slice.bytes.len() {
            tokenizer.events[exit_index - 1].name = Name::SPACE_OR_TAB;
            tokenizer.events[exit_index].name = Name::SPACE_OR_TAB;
            return;
        }
        if index > 0 || slice.before > 0 {
            let enter_point = tokenizer.events[exit_index - 1].point.clone();
            let mut exit_point = enter_point.clone();
            exit_point.index += index;
            exit_point.column += index;
            exit_point.vs = 0;
            tokenizer.map.add(
                exit_index - 1,
                0,
                vec![
                    Event {
                        kind: Kind::Enter,
                        name: Name::SPACE_OR_TAB,
                        point: enter_point,
                        link: None,
                    },
                    Event {
                        kind: Kind::Exit,
                        name: Name::SPACE_OR_TAB,
                        point: exit_point.clone(),
                        link: None,
                    },
                ],
            );
            tokenizer.events[exit_index - 1].point = exit_point;
        }
    }
}
