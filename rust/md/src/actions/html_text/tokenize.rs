use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab;
use crate::helpers::html::CDATA_PREFIX;
use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

// What a scan to the end without closing proves about later ones, by kind
const COMMENT: usize = 0;
const CDATA: usize = 1;
const DECLARATION: usize = 2;
const INSTRUCTION: usize = 3;

/// Inline HTML: a tag, comment, processing instruction, declaration or CDATA.
pub(crate) fn tokenize_html_text(tokenizer: &mut Tokenizer) -> State {
    if Some(b'<') == tokenizer.current && tokenizer.parse_state.mdx.is_none() {
        tokenizer.enter(Name::HTML_TEXT);
        tokenizer.enter(Name::HTML_TEXT_DATA);
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
            State::Next(tag_close_start)
        }
        Some(b'?') if !tokenizer.tokenize_state.html_text_unclosed[INSTRUCTION] => {
            tokenizer.consume();
            State::Next(instruction)
        }
        Some(b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(tag_open)
        }
        _ => State::Nok,
    }
}

fn declaration_open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-') => {
            tokenizer.consume();
            State::Next(comment_open_inside)
        }
        Some(b'A'..=b'Z' | b'a'..=b'z')
            if !tokenizer.tokenize_state.html_text_unclosed[DECLARATION] =>
        {
            tokenizer.consume();
            State::Next(declaration)
        }
        Some(b'[') => {
            tokenizer.consume();
            State::Next(cdata_open_inside)
        }
        _ => State::Nok,
    }
}

fn comment_open_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-') if !tokenizer.tokenize_state.html_text_unclosed[COMMENT] => {
            tokenizer.consume();
            State::Next(comment_end)
        }
        _ => State::Nok,
    }
}

fn comment(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => unclosed(tokenizer, COMMENT),
        Some(b'\n') => {
            tokenizer.attempt(State::Next(comment), State::Nok);
            State::Retry(line_ending_before)
        }
        Some(b'-') => {
            tokenizer.consume();
            State::Next(comment_close)
        }
        _ => {
            tokenizer.consume();
            State::Next(comment)
        }
    }
}

fn comment_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-') => {
            tokenizer.consume();
            State::Next(comment_end)
        }
        _ => State::Retry(comment),
    }
}
fn comment_end(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'>') => State::Retry(end),
        Some(b'-') => State::Retry(comment_close),
        _ => State::Retry(comment),
    }
}

fn cdata_open_inside(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(CDATA_PREFIX[tokenizer.tokenize_state.size]) {
        tokenizer.tokenize_state.size += 1;
        tokenizer.consume();

        if tokenizer.tokenize_state.size == CDATA_PREFIX.len() {
            tokenizer.tokenize_state.size = 0;
            if tokenizer.tokenize_state.html_text_unclosed[CDATA] {
                return State::Nok;
            }
            State::Next(cdata)
        } else {
            State::Next(cdata_open_inside)
        }
    } else {
        State::Nok
    }
}

fn cdata(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => unclosed(tokenizer, CDATA),
        Some(b'\n') => {
            tokenizer.attempt(State::Next(cdata), State::Nok);
            State::Retry(line_ending_before)
        }
        Some(b']') => {
            tokenizer.consume();
            State::Next(cdata_close)
        }
        _ => {
            tokenizer.consume();
            State::Next(cdata)
        }
    }
}

fn cdata_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b']') => {
            tokenizer.consume();
            State::Next(cdata_end)
        }
        _ => State::Retry(cdata),
    }
}

fn cdata_end(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'>') => State::Retry(end),
        Some(b']') => State::Retry(cdata_close),
        _ => State::Retry(cdata),
    }
}

fn declaration(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => unclosed(tokenizer, DECLARATION),
        Some(b'>') => State::Retry(end),
        Some(b'\n') => {
            tokenizer.attempt(State::Next(declaration), State::Nok);
            State::Retry(line_ending_before)
        }
        _ => {
            tokenizer.consume();
            State::Next(declaration)
        }
    }
}

fn instruction(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => unclosed(tokenizer, INSTRUCTION),
        Some(b'\n') => {
            tokenizer.attempt(State::Next(instruction), State::Nok);
            State::Retry(line_ending_before)
        }
        Some(b'?') => {
            tokenizer.consume();
            State::Next(instruction_close)
        }
        _ => {
            tokenizer.consume();
            State::Next(instruction)
        }
    }
}

fn instruction_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'>') => State::Retry(end),
        _ => State::Retry(instruction),
    }
}

fn tag_close_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(tag_close)
        }
        _ => State::Nok,
    }
}

fn tag_close(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(tag_close)
        }
        _ => State::Retry(tag_close_between),
    }
}

fn tag_close_between(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => {
            tokenizer.attempt(State::Next(tag_close_between), State::Nok);
            State::Retry(line_ending_before)
        }
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(tag_close_between)
        }
        _ => State::Retry(end),
    }
}

fn tag_open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(tag_open)
        }
        Some(b'\t' | b'\n' | b' ' | b'/' | b'>') => State::Retry(tag_open_between),
        _ => State::Nok,
    }
}

fn tag_open_between(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => {
            tokenizer.attempt(State::Next(tag_open_between), State::Nok);
            State::Retry(line_ending_before)
        }
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(tag_open_between)
        }
        Some(b'/') => {
            tokenizer.consume();
            State::Next(end)
        }
        Some(b':' | b'A'..=b'Z' | b'_' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(tag_open_attribute_name)
        }
        _ => State::Retry(end),
    }
}

fn tag_open_attribute_name(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-' | b'.' | b'0'..=b'9' | b':' | b'A'..=b'Z' | b'_' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(tag_open_attribute_name)
        }
        _ => State::Retry(tag_open_attribute_name_after),
    }
}

fn tag_open_attribute_name_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => {
            tokenizer.attempt(State::Next(tag_open_attribute_name_after), State::Nok);
            State::Retry(line_ending_before)
        }
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(tag_open_attribute_name_after)
        }
        Some(b'=') => {
            tokenizer.consume();
            State::Next(tag_open_attribute_value_before)
        }
        _ => State::Retry(tag_open_between),
    }
}

fn tag_open_attribute_value_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'<' | b'=' | b'>' | b'`') => State::Nok,
        Some(b'\n') => {
            tokenizer.attempt(State::Next(tag_open_attribute_value_before), State::Nok);
            State::Retry(line_ending_before)
        }
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(tag_open_attribute_value_before)
        }
        Some(b'"' | b'\'') => {
            tokenizer.tokenize_state.marker = tokenizer.current.unwrap_or_default();
            tokenizer.consume();
            State::Next(tag_open_attribute_value_quoted)
        }
        Some(_) => {
            tokenizer.consume();
            State::Next(tag_open_attribute_value_unquoted)
        }
    }
}

fn tag_open_attribute_value_quoted(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(tokenizer.tokenize_state.marker) {
        tokenizer.tokenize_state.marker = 0;
        tokenizer.consume();
        State::Next(tag_open_attribute_value_quoted_after)
    } else {
        match tokenizer.current {
            None => {
                tokenizer.tokenize_state.marker = 0;
                State::Nok
            }
            Some(b'\n') => {
                tokenizer
                    .attempt(State::Next(tag_open_attribute_value_quoted), State::Nok);
                State::Retry(line_ending_before)
            }
            _ => {
                tokenizer.consume();
                State::Next(tag_open_attribute_value_quoted)
            }
        }
    }
}

fn tag_open_attribute_value_unquoted(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'"' | b'\'' | b'<' | b'=' | b'`') => State::Nok,
        Some(b'\t' | b'\n' | b' ' | b'/' | b'>') => State::Retry(tag_open_between),
        Some(_) => {
            tokenizer.consume();
            State::Next(tag_open_attribute_value_unquoted)
        }
    }
}

fn tag_open_attribute_value_quoted_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b'\n' | b' ' | b'/' | b'>') => State::Retry(tag_open_between),
        _ => State::Nok,
    }
}

fn end(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'>') => {
            tokenizer.consume();
            tokenizer.exit(Name::HTML_TEXT_DATA);
            tokenizer.exit(Name::HTML_TEXT);
            State::Ok
        }
        _ => State::Nok,
    }
}

fn line_ending_before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => {
            tokenizer.exit(Name::HTML_TEXT_DATA);
            tokenizer.enter(Name::LINE_ENDING);
            tokenizer.consume();
            tokenizer.exit(Name::LINE_ENDING);
            State::Next(line_ending_after)
        }
        _ => State::Nok,
    }
}

fn line_ending_after(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.attempt(State::Next(line_ending_after_prefix), State::Nok);
        State::Retry(tokenize_space_or_tab(tokenizer, 1, usize::MAX))
    } else {
        State::Retry(line_ending_after_prefix)
    }
}

fn line_ending_after_prefix(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::HTML_TEXT_DATA);
    State::Ok
}
// No closing of this kind follows: later ones fail at once instead of scanning again, which
// would make many unclosed comments quadratic
fn unclosed(tokenizer: &mut Tokenizer, kind: usize) -> State {
    tokenizer.tokenize_state.html_text_unclosed[kind] = true;
    State::Nok
}
