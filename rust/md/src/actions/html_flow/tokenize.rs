use crate::actions::blank_line::tokenize::tokenize_blank_line;
use crate::actions::non_lazy_continuation::tokenize::tokenize_non_lazy_continuation;
use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab_named;
use crate::helpers::html::{BLOCK_NAMES, CDATA_PREFIX, RAW_NAMES, RAW_SIZE_MAX};
use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const TAB_SIZE: usize = 4;
// The kinds of HTML block, by what ends them
const RAW: u8 = 1;
const COMMENT: u8 = 2;
const INSTRUCTION: u8 = 3;
const DECLARATION: u8 = 4;
const CDATA: u8 = 5;
const BASIC: u8 = 6;
const COMPLETE: u8 = 7;

/// An HTML block: one of seven kinds by how it starts, each ending its own way.
pub(crate) fn tokenize_html_flow(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.mdx.is_none() {
        tokenizer.enter(Name::HTML_FLOW);

        if matches!(tokenizer.current, Some(b'\t' | b' ')) {
            tokenizer.attempt(State::Next(before), State::Nok);
            State::Retry(tokenize_space_or_tab_named(
                tokenizer,
                Name::HTML_FLOW_DATA,
                0,
                TAB_SIZE - 1,
            ))
        } else {
            State::Retry(before)
        }
    } else {
        State::Nok
    }
}

fn before(tokenizer: &mut Tokenizer) -> State {
    if Some(b'<') == tokenizer.current {
        tokenizer.enter(Name::HTML_FLOW_DATA);
        tokenizer.consume();
        State::Next(open)
    } else {
        State::Nok
    }
}

fn open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'!') => {
            tokenizer.consume();
            State::Next(declaration_open)
        }
        Some(b'/') => {
            tokenizer.consume();
            tokenizer.tokenize_state.seen = true;
            tokenizer.tokenize_state.start = tokenizer.point.index as usize;
            State::Next(tag_close_start)
        }
        Some(b'?') => {
            tokenizer.consume();
            tokenizer.tokenize_state.marker = INSTRUCTION;
            tokenizer.concrete = true;
            State::Next(continuation_declaration_inside)
        }
        Some(b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.tokenize_state.start = tokenizer.point.index as usize;
            State::Retry(tag_name)
        }
        _ => State::Nok,
    }
}

fn declaration_open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-') => {
            tokenizer.consume();
            tokenizer.tokenize_state.marker = COMMENT;
            State::Next(comment_open_inside)
        }
        Some(b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.consume();
            tokenizer.tokenize_state.marker = DECLARATION;
            tokenizer.concrete = true;
            State::Next(continuation_declaration_inside)
        }
        Some(b'[') => {
            tokenizer.consume();
            tokenizer.tokenize_state.marker = CDATA;
            State::Next(cdata_open_inside)
        }
        _ => State::Nok,
    }
}

fn comment_open_inside(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'-') = tokenizer.current {
        tokenizer.consume();
        tokenizer.concrete = true;
        State::Next(continuation_declaration_inside)
    } else {
        tokenizer.tokenize_state.marker = 0;
        State::Nok
    }
}

fn cdata_open_inside(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(CDATA_PREFIX[tokenizer.tokenize_state.size]) {
        tokenizer.consume();
        tokenizer.tokenize_state.size += 1;

        if tokenizer.tokenize_state.size == CDATA_PREFIX.len() {
            tokenizer.tokenize_state.size = 0;
            tokenizer.concrete = true;
            State::Next(continuation)
        } else {
            State::Next(cdata_open_inside)
        }
    } else {
        tokenizer.tokenize_state.marker = 0;
        tokenizer.tokenize_state.size = 0;
        State::Nok
    }
}

fn tag_close_start(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'A'..=b'Z' | b'a'..=b'z') = tokenizer.current {
        tokenizer.consume();
        State::Next(tag_name)
    } else {
        tokenizer.tokenize_state.seen = false;
        tokenizer.tokenize_state.start = 0;
        State::Nok
    }
}

fn tag_name(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\t' | b'\n' | b' ' | b'/' | b'>') => {
            let closing_tag = tokenizer.tokenize_state.seen;
            let slash = matches!(tokenizer.current, Some(b'/'));
            let name = read_name(tokenizer);
            tokenizer.tokenize_state.seen = false;
            tokenizer.tokenize_state.start = 0;

            if !slash && !closing_tag && RAW_NAMES.contains(&name.as_str()) {
                tokenizer.tokenize_state.marker = RAW;
                tokenizer.concrete = true;
                State::Retry(continuation)
            } else if BLOCK_NAMES.contains(&name.as_str()) {
                tokenizer.tokenize_state.marker = BASIC;

                if slash {
                    tokenizer.consume();
                    State::Next(basic_self_closing)
                } else {
                    tokenizer.concrete = true;
                    State::Retry(continuation)
                }
            } else {
                tokenizer.tokenize_state.marker = COMPLETE;

                if tokenizer.interrupt && !tokenizer.lazy {
                    tokenizer.tokenize_state.marker = 0;
                    State::Nok
                } else if closing_tag {
                    State::Retry(complete_closing_tag_after)
                } else {
                    State::Retry(complete_attribute_name_before)
                }
            }
        }
        Some(b'-' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(tag_name)
        }
        Some(_) => {
            tokenizer.tokenize_state.seen = false;
            State::Nok
        }
    }
}

fn basic_self_closing(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'>') = tokenizer.current {
        tokenizer.consume();
        tokenizer.concrete = true;
        State::Next(continuation)
    } else {
        tokenizer.tokenize_state.marker = 0;
        State::Nok
    }
}

fn complete_closing_tag_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(complete_closing_tag_after)
        }
        _ => State::Retry(complete_end),
    }
}

fn complete_attribute_name_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(complete_attribute_name_before)
        }
        Some(b'/') => {
            tokenizer.consume();
            State::Next(complete_end)
        }
        Some(b'0'..=b'9' | b':' | b'A'..=b'Z' | b'_' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(complete_attribute_name)
        }
        _ => State::Retry(complete_end),
    }
}

fn complete_attribute_name(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-' | b'.' | b'0'..=b'9' | b':' | b'A'..=b'Z' | b'_' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(complete_attribute_name)
        }
        _ => State::Retry(complete_attribute_name_after),
    }
}

fn complete_attribute_name_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(complete_attribute_name_after)
        }
        Some(b'=') => {
            tokenizer.consume();
            State::Next(complete_attribute_value_before)
        }
        _ => State::Retry(complete_attribute_name_before),
    }
}

fn complete_attribute_value_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'<' | b'=' | b'>' | b'`') => {
            tokenizer.tokenize_state.marker = 0;
            State::Nok
        }
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(complete_attribute_value_before)
        }
        Some(b'"' | b'\'') => {
            tokenizer.tokenize_state.marker_b = tokenizer.current.unwrap_or_default();
            tokenizer.consume();
            State::Next(complete_attribute_value_quoted)
        }
        _ => State::Retry(complete_attribute_value_unquoted),
    }
}

fn complete_attribute_value_quoted(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(tokenizer.tokenize_state.marker_b) {
        tokenizer.consume();
        tokenizer.tokenize_state.marker_b = 0;
        State::Next(complete_attribute_value_quoted_after)
    } else if matches!(tokenizer.current, None | Some(b'\n')) {
        tokenizer.tokenize_state.marker = 0;
        tokenizer.tokenize_state.marker_b = 0;
        State::Nok
    } else {
        tokenizer.consume();
        State::Next(complete_attribute_value_quoted)
    }
}

fn complete_attribute_value_unquoted(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None
        | Some(b'\t' | b'\n' | b' ' | b'"' | b'\'' | b'/' | b'<' | b'=' | b'>' | b'`') => {
            State::Retry(complete_attribute_name_after)
        }
        Some(_) => {
            tokenizer.consume();
            State::Next(complete_attribute_value_unquoted)
        }
    }
}

fn complete_attribute_value_quoted_after(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'\t' | b' ' | b'/' | b'>') = tokenizer.current {
        State::Retry(complete_attribute_name_before)
    } else {
        tokenizer.tokenize_state.marker = 0;
        State::Nok
    }
}

fn complete_end(tokenizer: &mut Tokenizer) -> State {
    if let Some(b'>') = tokenizer.current {
        tokenizer.consume();
        State::Next(complete_after)
    } else {
        tokenizer.tokenize_state.marker = 0;
        State::Nok
    }
}

fn complete_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.concrete = true;
            State::Retry(continuation)
        }
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(complete_after)
        }
        Some(_) => {
            tokenizer.tokenize_state.marker = 0;
            State::Nok
        }
    }
}

fn continuation(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.tokenize_state.marker == COMMENT && tokenizer.current == Some(b'-') {
        tokenizer.consume();
        State::Next(continuation_comment_inside)
    } else if tokenizer.tokenize_state.marker == RAW && tokenizer.current == Some(b'<') {
        tokenizer.consume();
        State::Next(continuation_raw_tag_open)
    } else if tokenizer.tokenize_state.marker == DECLARATION
        && tokenizer.current == Some(b'>')
    {
        tokenizer.consume();
        State::Next(continuation_close)
    } else if tokenizer.tokenize_state.marker == INSTRUCTION
        && tokenizer.current == Some(b'?')
    {
        tokenizer.consume();
        State::Next(continuation_declaration_inside)
    } else if tokenizer.tokenize_state.marker == CDATA && tokenizer.current == Some(b']')
    {
        tokenizer.consume();
        State::Next(continuation_cdata_inside)
    } else if matches!(tokenizer.tokenize_state.marker, BASIC | COMPLETE)
        && tokenizer.current == Some(b'\n')
    {
        tokenizer.exit(Name::HTML_FLOW_DATA);
        tokenizer.check(
            State::Next(continuation_after),
            State::Next(continuation_start),
        );
        State::Retry(blank_line_before)
    } else if matches!(tokenizer.current, None | Some(b'\n')) {
        tokenizer.exit(Name::HTML_FLOW_DATA);
        State::Retry(continuation_start)
    } else {
        tokenizer.consume();
        State::Next(continuation)
    }
}

fn continuation_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.check(
        State::Next(continuation_start_non_lazy),
        State::Next(continuation_after),
    );
    State::Retry(tokenize_non_lazy_continuation)
}

fn continuation_start_non_lazy(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => {
            tokenizer.enter(Name::LINE_ENDING);
            tokenizer.consume();
            tokenizer.exit(Name::LINE_ENDING);
            State::Next(continuation_before)
        }
        _ => State::Nok,
    }
}

fn continuation_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => State::Retry(continuation_start),
        _ => {
            tokenizer.enter(Name::HTML_FLOW_DATA);
            State::Retry(continuation)
        }
    }
}

fn continuation_comment_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-') => {
            tokenizer.consume();
            State::Next(continuation_declaration_inside)
        }
        _ => State::Retry(continuation),
    }
}

fn continuation_raw_tag_open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'/') => {
            tokenizer.consume();
            tokenizer.tokenize_state.start = tokenizer.point.index as usize;
            State::Next(continuation_raw_end_tag)
        }
        _ => State::Retry(continuation),
    }
}

fn continuation_raw_end_tag(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'>') => {
            let name = read_name(tokenizer);

            tokenizer.tokenize_state.start = 0;

            if RAW_NAMES.contains(&name.as_str()) {
                tokenizer.consume();
                State::Next(continuation_close)
            } else {
                State::Retry(continuation)
            }
        }
        Some(b'A'..=b'Z' | b'a'..=b'z')
            if (tokenizer.point.index as usize) - tokenizer.tokenize_state.start
                < RAW_SIZE_MAX =>
        {
            tokenizer.consume();
            State::Next(continuation_raw_end_tag)
        }
        _ => {
            tokenizer.tokenize_state.start = 0;
            State::Retry(continuation)
        }
    }
}

fn continuation_cdata_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b']') => {
            tokenizer.consume();
            State::Next(continuation_declaration_inside)
        }
        _ => State::Retry(continuation),
    }
}

fn continuation_declaration_inside(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.tokenize_state.marker == COMMENT && tokenizer.current == Some(b'-') {
        tokenizer.consume();
        State::Next(continuation_declaration_inside)
    } else if tokenizer.current == Some(b'>') {
        tokenizer.consume();
        State::Next(continuation_close)
    } else {
        State::Retry(continuation)
    }
}

fn continuation_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::HTML_FLOW_DATA);
            State::Retry(continuation_after)
        }
        _ => {
            tokenizer.consume();
            State::Next(continuation_close)
        }
    }
}

fn continuation_after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(Name::HTML_FLOW);
    tokenizer.tokenize_state.marker = 0;
    tokenizer.interrupt = false;
    tokenizer.concrete = false;
    State::Ok
}

fn blank_line_before(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::LINE_ENDING);
    tokenizer.consume();
    tokenizer.exit(Name::LINE_ENDING);
    State::Next(tokenize_blank_line)
}
// The tag name from `start` to here, lowercase
fn read_name(tokenizer: &Tokenizer) -> String {
    let bytes = tokenizer.parse_state.bytes;
    let name = bytes
        .get(tokenizer.tokenize_state.start..tokenizer.point.index as usize)
        .unwrap_or_default();
    String::from_utf8_lossy(name).trim().to_ascii_lowercase()
}
