use textlane_core::Error;

use crate::actions::mdx_expression::tokenize::tokenize_mdx_expression;
use crate::helpers::character::find_character_after;
use crate::helpers::script::{
    format_character, is_id_cont, is_id_start, is_js_whitespace,
};
use crate::models::construct::{State, StateFn};
use crate::models::mdx::ES_WHITESPACE;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const SOURCE: &str = "micromark-extension-mdx-jsx";

/// A JSX tag, `<a b="c">`, `</a>` or `<a />`, as micromark-extension-mdx-jsx's tag factory:
/// its token names and whether lazy lines may continue it are set by the caller.
pub(crate) fn tokenize_mdx_jsx(tokenizer: &mut Tokenizer) -> State {
    let names = tokenizer.tokenize_state.mdx.jsx;
    tokenizer.enter(names.tag);
    tokenizer.enter(names.marker);
    tokenizer.consume();
    tokenizer.exit(names.marker);
    State::Next(start_after)
}

fn start_after(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b'\n' | b' ')) {
        return State::Nok;
    }
    es_whitespace(tokenizer, name_before)
}

fn name_before(tokenizer: &mut Tokenizer) -> State {
    let names = tokenizer.tokenize_state.mdx.jsx;
    match tokenizer.current {
        Some(b'/') => {
            tokenizer.enter(names.closing_marker);
            tokenizer.consume();
            tokenizer.exit(names.closing_marker);
            tokenizer.tokenize_state.mdx.jsx_return = closing_tag_name_before;
            State::Next(es_whitespace_start)
        }
        Some(b'>') => State::Retry(tag_end),
        _ if starts_id(tokenizer) => {
            tokenizer.enter(names.name);
            tokenizer.enter(names.name_primary);
            tokenizer.consume();
            State::Next(primary_name)
        }
        current => {
            let note = if current == Some(b'!') {
                " (note: to create a comment in MDX, use `{/* text */}`)"
            } else {
                ""
            };
            crash(
                tokenizer,
                "before name",
                &format!(
                    "a character that can start a name, such as a letter, `$`, or `_`{note}"
                ),
            )
        }
    }
}

fn closing_tag_name_before(tokenizer: &mut Tokenizer) -> State {
    let names = tokenizer.tokenize_state.mdx.jsx;
    match tokenizer.current {
        Some(b'>') => State::Retry(tag_end),
        _ if starts_id(tokenizer) => {
            tokenizer.enter(names.name);
            tokenizer.enter(names.name_primary);
            tokenizer.consume();
            State::Next(primary_name)
        }
        current => {
            let note = if matches!(current, Some(b'*' | b'/')) {
                " (note: JS comments in JSX tags are not supported in MDX)"
            } else {
                ""
            };
            crash(
                tokenizer,
                "before name",
                &format!(
                    "a character that can start a name, such as a letter, `$`, or `_`{note}"
                ),
            )
        }
    }
}

fn primary_name(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'.' | b'/' | b':' | b'>' | b'{'))
        || is_whitespace(tokenizer)
    {
        tokenizer.exit(tokenizer.tokenize_state.mdx.jsx.name_primary);
        return es_whitespace(tokenizer, primary_name_after);
    }
    if continues_id(tokenizer) {
        tokenizer.consume();
        return State::Next(primary_name);
    }
    let note = link_note(tokenizer);
    crash(
        tokenizer,
        "in name",
        &format!(
            "a name character such as letters, digits, `$`, or `_`; whitespace before attributes; or the end of the tag{note}"
        ),
    )
}

fn primary_name_after(tokenizer: &mut Tokenizer) -> State {
    let names = tokenizer.tokenize_state.mdx.jsx;
    match tokenizer.current {
        Some(b'.') => {
            tokenizer.enter(names.name_member_marker);
            tokenizer.consume();
            tokenizer.exit(names.name_member_marker);
            tokenizer.tokenize_state.mdx.jsx_return = member_name_before;
            State::Next(es_whitespace_start)
        }
        Some(b':') => {
            tokenizer.enter(names.name_prefix_marker);
            tokenizer.consume();
            tokenizer.exit(names.name_prefix_marker);
            tokenizer.tokenize_state.mdx.jsx_return = local_name_before;
            State::Next(es_whitespace_start)
        }
        Some(b'/' | b'>' | b'{') => {
            tokenizer.exit(names.name);
            State::Retry(attribute_before)
        }
        _ if starts_id(tokenizer) => {
            tokenizer.exit(names.name);
            State::Retry(attribute_before)
        }
        _ => crash(
            tokenizer,
            "after name",
            "a character that can start an attribute name, such as a letter, `$`, or `_`; whitespace before attributes; or the end of the tag",
        ),
    }
}

fn member_name_before(tokenizer: &mut Tokenizer) -> State {
    if starts_id(tokenizer) {
        tokenizer.enter(tokenizer.tokenize_state.mdx.jsx.name_member);
        tokenizer.consume();
        return State::Next(member_name);
    }
    crash(
        tokenizer,
        "before member name",
        "a character that can start an attribute name, such as a letter, `$`, or `_`; whitespace before attributes; or the end of the tag",
    )
}

fn member_name(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'.' | b'/' | b'>' | b'{'))
        || is_whitespace(tokenizer)
    {
        tokenizer.exit(tokenizer.tokenize_state.mdx.jsx.name_member);
        return es_whitespace(tokenizer, member_name_after);
    }
    if continues_id(tokenizer) {
        tokenizer.consume();
        return State::Next(member_name);
    }
    let note = link_note(tokenizer);
    crash(
        tokenizer,
        "in member name",
        &format!(
            "a name character such as letters, digits, `$`, or `_`; whitespace before attributes; or the end of the tag{note}"
        ),
    )
}

fn member_name_after(tokenizer: &mut Tokenizer) -> State {
    let names = tokenizer.tokenize_state.mdx.jsx;
    match tokenizer.current {
        Some(b'.') => {
            tokenizer.enter(names.name_member_marker);
            tokenizer.consume();
            tokenizer.exit(names.name_member_marker);
            tokenizer.tokenize_state.mdx.jsx_return = member_name_before;
            State::Next(es_whitespace_start)
        }
        Some(b'/' | b'>' | b'{') => {
            tokenizer.exit(names.name);
            State::Retry(attribute_before)
        }
        _ if starts_id(tokenizer) => {
            tokenizer.exit(names.name);
            State::Retry(attribute_before)
        }
        _ => crash(
            tokenizer,
            "after member name",
            "a character that can start an attribute name, such as a letter, `$`, or `_`; whitespace before attributes; or the end of the tag",
        ),
    }
}

fn local_name_before(tokenizer: &mut Tokenizer) -> State {
    if starts_id(tokenizer) {
        tokenizer.enter(tokenizer.tokenize_state.mdx.jsx.name_local);
        tokenizer.consume();
        return State::Next(local_name);
    }
    let note = if matches!(tokenizer.current, Some(b'+' | b'/'..=b'9')) {
        " (note: to create a link in MDX, use `[text](url)`)"
    } else {
        ""
    };
    crash(
        tokenizer,
        "before local name",
        &format!(
            "a character that can start a name, such as a letter, `$`, or `_`{note}"
        ),
    )
}

fn local_name(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'/' | b'>' | b'{')) || is_whitespace(tokenizer) {
        tokenizer.exit(tokenizer.tokenize_state.mdx.jsx.name_local);
        return es_whitespace(tokenizer, local_name_after);
    }
    if continues_id(tokenizer) {
        tokenizer.consume();
        return State::Next(local_name);
    }
    crash(
        tokenizer,
        "in local name",
        "a name character such as letters, digits, `$`, or `_`; whitespace before attributes; or the end of the tag",
    )
}

fn local_name_after(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'/' | b'>' | b'{')) || starts_id(tokenizer) {
        tokenizer.exit(tokenizer.tokenize_state.mdx.jsx.name);
        return State::Retry(attribute_before);
    }
    crash(
        tokenizer,
        "after local name",
        "a character that can start an attribute name, such as a letter, `$`, or `_`; whitespace before attributes; or the end of the tag",
    )
}

fn attribute_before(tokenizer: &mut Tokenizer) -> State {
    let names = tokenizer.tokenize_state.mdx.jsx;
    match tokenizer.current {
        Some(b'/') => {
            tokenizer.enter(names.self_closing_marker);
            tokenizer.consume();
            tokenizer.exit(names.self_closing_marker);
            tokenizer.tokenize_state.mdx.jsx_return = self_closing;
            State::Next(es_whitespace_start)
        }
        Some(b'>') => State::Retry(tag_end),
        Some(b'{') => {
            let state = &mut tokenizer.tokenize_state.mdx;
            state.expression = &names.expression_attribute;
            state.spread = true;
            state.allow_empty = false;
            tokenizer.attempt(State::Next(attribute_expression_after), State::Nok);
            State::Retry(tokenize_mdx_expression)
        }
        _ if starts_id(tokenizer) => {
            tokenizer.enter(names.attribute);
            tokenizer.enter(names.attribute_name);
            tokenizer.enter(names.attribute_name_primary);
            tokenizer.consume();
            State::Next(attribute_primary_name)
        }
        _ => crash(
            tokenizer,
            "before attribute name",
            "a character that can start an attribute name, such as a letter, `$`, or `_`; whitespace before attributes; or the end of the tag",
        ),
    }
}

fn attribute_expression_after(tokenizer: &mut Tokenizer) -> State {
    es_whitespace(tokenizer, attribute_before)
}

fn attribute_primary_name(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'/' | b':' | b'=' | b'>' | b'{'))
        || is_whitespace(tokenizer)
    {
        tokenizer.exit(tokenizer.tokenize_state.mdx.jsx.attribute_name_primary);
        return es_whitespace(tokenizer, attribute_primary_name_after);
    }
    if continues_id(tokenizer) {
        tokenizer.consume();
        return State::Next(attribute_primary_name);
    }
    crash(
        tokenizer,
        "in attribute name",
        "an attribute name character such as letters, digits, `$`, or `_`; `=` to initialize a value; whitespace before attributes; or the end of the tag",
    )
}

fn attribute_primary_name_after(tokenizer: &mut Tokenizer) -> State {
    let names = tokenizer.tokenize_state.mdx.jsx;
    match tokenizer.current {
        Some(b':') => {
            tokenizer.enter(names.attribute_name_prefix_marker);
            tokenizer.consume();
            tokenizer.exit(names.attribute_name_prefix_marker);
            tokenizer.tokenize_state.mdx.jsx_return = attribute_local_name_before;
            State::Next(es_whitespace_start)
        }
        Some(b'=') => {
            tokenizer.exit(names.attribute_name);
            tokenizer.enter(names.attribute_initializer_marker);
            tokenizer.consume();
            tokenizer.exit(names.attribute_initializer_marker);
            tokenizer.tokenize_state.mdx.jsx_return = attribute_value_before;
            State::Next(es_whitespace_start)
        }
        Some(b'/' | b'>' | b'{') => {
            tokenizer.exit(names.attribute_name);
            tokenizer.exit(names.attribute);
            es_whitespace(tokenizer, attribute_before)
        }
        _ if is_whitespace(tokenizer) || starts_id(tokenizer) => {
            tokenizer.exit(names.attribute_name);
            tokenizer.exit(names.attribute);
            es_whitespace(tokenizer, attribute_before)
        }
        _ => crash(
            tokenizer,
            "after attribute name",
            "a character that can start an attribute name, such as a letter, `$`, or `_`; `=` to initialize a value; or the end of the tag",
        ),
    }
}

fn attribute_local_name_before(tokenizer: &mut Tokenizer) -> State {
    if starts_id(tokenizer) {
        tokenizer.enter(tokenizer.tokenize_state.mdx.jsx.attribute_name_local);
        tokenizer.consume();
        return State::Next(attribute_local_name);
    }
    crash(
        tokenizer,
        "before local attribute name",
        "a character that can start an attribute name, such as a letter, `$`, or `_`; `=` to initialize a value; or the end of the tag",
    )
}

fn attribute_local_name(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'/' | b'=' | b'>' | b'{'))
        || is_whitespace(tokenizer)
    {
        let names = tokenizer.tokenize_state.mdx.jsx;
        tokenizer.exit(names.attribute_name_local);
        tokenizer.exit(names.attribute_name);
        return es_whitespace(tokenizer, attribute_local_name_after);
    }
    if continues_id(tokenizer) {
        tokenizer.consume();
        return State::Next(attribute_local_name);
    }
    crash(
        tokenizer,
        "in local attribute name",
        "an attribute name character such as letters, digits, `$`, or `_`; `=` to initialize a value; whitespace before attributes; or the end of the tag",
    )
}

fn attribute_local_name_after(tokenizer: &mut Tokenizer) -> State {
    let names = tokenizer.tokenize_state.mdx.jsx;
    match tokenizer.current {
        Some(b'=') => {
            tokenizer.enter(names.attribute_initializer_marker);
            tokenizer.consume();
            tokenizer.exit(names.attribute_initializer_marker);
            tokenizer.tokenize_state.mdx.jsx_return = attribute_value_before;
            State::Next(es_whitespace_start)
        }
        Some(b'/' | b'>' | b'{') => {
            tokenizer.exit(names.attribute);
            State::Retry(attribute_before)
        }
        _ if starts_id(tokenizer) => {
            tokenizer.exit(names.attribute);
            State::Retry(attribute_before)
        }
        _ => crash(
            tokenizer,
            "after local attribute name",
            "a character that can start an attribute name, such as a letter, `$`, or `_`; `=` to initialize a value; or the end of the tag",
        ),
    }
}

fn attribute_value_before(tokenizer: &mut Tokenizer) -> State {
    let names = tokenizer.tokenize_state.mdx.jsx;
    match tokenizer.current {
        Some(byte @ (b'"' | b'\'')) => {
            tokenizer.enter(names.attribute_value_literal);
            tokenizer.enter(names.attribute_value_literal_marker);
            tokenizer.consume();
            tokenizer.exit(names.attribute_value_literal_marker);
            tokenizer.tokenize_state.mdx.jsx_marker = byte;
            State::Next(attribute_value_quoted_start)
        }
        Some(b'{') => {
            let state = &mut tokenizer.tokenize_state.mdx;
            state.expression = &names.attribute_value_expression;
            state.spread = false;
            state.allow_empty = false;
            tokenizer.attempt(State::Next(attribute_value_expression_after), State::Nok);
            State::Retry(tokenize_mdx_expression)
        }
        current => {
            let note = if current == Some(b'<') {
                " (note: to use an element or fragment as a prop value in MDX, use `{<element />}`)"
            } else {
                ""
            };
            crash(
                tokenizer,
                "before attribute value",
                &format!(
                    "a character that can start an attribute value, such as `\"`, `'`, or `{{`{note}"
                ),
            )
        }
    }
}

fn attribute_value_expression_after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(tokenizer.tokenize_state.mdx.jsx.attribute);
    es_whitespace(tokenizer, attribute_before)
}

fn attribute_value_quoted_start(tokenizer: &mut Tokenizer) -> State {
    let names = tokenizer.tokenize_state.mdx.jsx;
    let marker = tokenizer.tokenize_state.mdx.jsx_marker;
    match tokenizer.current {
        None => {
            let quote = char::from(marker);
            crash(
                tokenizer,
                "in attribute value",
                &format!("a corresponding closing quote `{quote}`"),
            )
        }
        Some(byte) if byte == marker => {
            tokenizer.enter(names.attribute_value_literal_marker);
            tokenizer.consume();
            tokenizer.exit(names.attribute_value_literal_marker);
            tokenizer.exit(names.attribute_value_literal);
            tokenizer.exit(names.attribute);
            tokenizer.tokenize_state.mdx.jsx_return = attribute_before;
            State::Next(es_whitespace_start)
        }
        Some(b'\n') => es_whitespace(tokenizer, attribute_value_quoted_start),
        Some(_) => {
            tokenizer.enter(names.attribute_value_literal_value);
            State::Retry(attribute_value_quoted)
        }
    }
}

fn attribute_value_quoted(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(byte)
            if byte != b'\n' && byte != tokenizer.tokenize_state.mdx.jsx_marker =>
        {
            tokenizer.consume();
            State::Next(attribute_value_quoted)
        }
        _ => {
            tokenizer.exit(
                tokenizer
                    .tokenize_state
                    .mdx
                    .jsx
                    .attribute_value_literal_value,
            );
            State::Retry(attribute_value_quoted_start)
        }
    }
}

fn self_closing(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'>') {
        return State::Retry(tag_end);
    }
    let note = if matches!(tokenizer.current, Some(b'*' | b'/')) {
        " (note: JS comments in JSX tags are not supported in MDX)"
    } else {
        ""
    };
    crash(
        tokenizer,
        "after self-closing slash",
        &format!("`>` to end the tag{note}"),
    )
}

fn tag_end(tokenizer: &mut Tokenizer) -> State {
    let names = tokenizer.tokenize_state.mdx.jsx;
    tokenizer.enter(names.marker);
    tokenizer.consume();
    tokenizer.exit(names.marker);
    tokenizer.exit(names.tag);
    State::Ok
}

// Skip ECMAScript whitespace, line endings included, then go to `next`
fn es_whitespace(tokenizer: &mut Tokenizer, next: StateFn) -> State {
    tokenizer.tokenize_state.mdx.jsx_return = next;
    State::Retry(es_whitespace_start)
}

fn es_whitespace_start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'\n') {
        tokenizer.enter(Name::LINE_ENDING);
        tokenizer.consume();
        tokenizer.exit(Name::LINE_ENDING);
        return State::Next(es_whitespace_eol_after);
    }
    if is_whitespace(tokenizer) {
        tokenizer.enter(ES_WHITESPACE);
        return State::Retry(es_whitespace_inside);
    }
    State::Retry(tokenizer.tokenize_state.mdx.jsx_return)
}

fn es_whitespace_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\n') => {
            tokenizer.exit(ES_WHITESPACE);
            State::Retry(es_whitespace_start)
        }
        Some(0x80..=0xBF) => {
            tokenizer.consume();
            State::Next(es_whitespace_inside)
        }
        _ if is_whitespace(tokenizer) => {
            tokenizer.consume();
            State::Next(es_whitespace_inside)
        }
        _ => {
            tokenizer.exit(ES_WHITESPACE);
            State::Retry(tokenizer.tokenize_state.mdx.jsx_return)
        }
    }
}

fn es_whitespace_eol_after(tokenizer: &mut Tokenizer) -> State {
    if !tokenizer.tokenize_state.mdx.allow_lazy && tokenizer.lazy {
        return tokenizer.crash(Error::new(
            format!("{SOURCE}:unexpected-lazy"),
            "Unexpected lazy line in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc",
        ));
    }
    State::Retry(es_whitespace_start)
}

fn current_character(tokenizer: &Tokenizer) -> Option<char> {
    match tokenizer.current {
        None => None,
        Some(byte) if byte.is_ascii() => Some(char::from(byte)),
        Some(_) => {
            find_character_after(tokenizer.parse_state.bytes, tokenizer.point.index)
        }
    }
}

fn starts_id(tokenizer: &Tokenizer) -> bool {
    current_character(tokenizer).is_some_and(is_id_start)
}

// The bytes after a character's first continue it
fn continues_id(tokenizer: &Tokenizer) -> bool {
    matches!(tokenizer.current, Some(0x80..=0xBF))
        || current_character(tokenizer).is_some_and(is_id_cont)
}

fn is_whitespace(tokenizer: &Tokenizer) -> bool {
    current_character(tokenizer).is_some_and(is_js_whitespace)
}

fn link_note(tokenizer: &Tokenizer) -> &'static str {
    if tokenizer.current == Some(b'@') {
        " (note: to create a link in MDX, use `[text](url)`)"
    } else {
        ""
    }
}

fn crash(tokenizer: &mut Tokenizer, at: &str, expect: &str) -> State {
    let character = current_character(tokenizer);
    let rule = if character.is_none() {
        "unexpected-eof"
    } else {
        "unexpected-character"
    };
    let reason = format!(
        "Unexpected {} {at}, expected {expect}",
        format_character(character)
    );
    tokenizer.crash(Error::new(format!("{SOURCE}:{rule}"), reason))
}
