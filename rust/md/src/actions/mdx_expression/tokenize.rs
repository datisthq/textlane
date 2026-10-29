use textlane_core::Error;

use crate::helpers::script::{append_script_source, is_empty_script};
use crate::models::construct::State;
use crate::models::mdx::LINE_PREFIX;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const SOURCE: &str = "micromark-extension-mdx-expression";
// micromark-factory-mdx-expression eats at most this much indent of each continuation line
const INDENT_SIZE: usize = 2;

/// An expression in braces, `{a}`, as micromark-factory-mdx-expression: its token names,
/// whether it must be a spread, may be empty and may span lazy lines are set by the caller.
/// With a script parser each `}` ends it only where the value before parses, as acorn
/// decides upstream; without one, braces balance.
pub(crate) fn tokenize_mdx_expression(tokenizer: &mut Tokenizer) -> State {
    let names = tokenizer.tokenize_state.mdx.expression;
    tokenizer.enter(names.expression);
    tokenizer.enter(names.marker);
    tokenizer.consume();
    tokenizer.exit(names.marker);
    let line = tokenizer.point.line;
    let events = tokenizer.events.len();
    let state = &mut tokenizer.tokenize_state.mdx;
    state.size = 0;
    state.line = line;
    state.events_start = events;
    state.events_read = events;
    state.source.clear();
    state.crash = None;
    state.hard_crash = false;
    state.parsed = 0;
    State::Next(before)
}

fn before(tokenizer: &mut Tokenizer) -> State {
    let names = tokenizer.tokenize_state.mdx.expression;
    match tokenizer.current {
        None => {
            let crash = tokenizer.tokenize_state.mdx.crash.take();
            tokenizer.crash(crash.unwrap_or_else(|| {
                Error::new(
                    format!("{SOURCE}:unexpected-eof"),
                    "Unexpected end of file in expression, expected a corresponding closing brace for `{`",
                )
            }))
        }
        Some(b'\n') => {
            tokenizer.enter(Name::LINE_ENDING);
            tokenizer.consume();
            tokenizer.exit(Name::LINE_ENDING);
            State::Next(eol_after)
        }
        Some(b'}') if tokenizer.tokenize_state.mdx.size == 0 => {
            match try_end(tokenizer) {
                Ok(true) => {
                    tokenizer.enter(names.marker);
                    tokenizer.consume();
                    tokenizer.exit(names.marker);
                    tokenizer.exit(names.expression);
                    State::Ok
                }
                Ok(false) => {
                    tokenizer.enter(names.chunk);
                    tokenizer.consume();
                    State::Next(inside)
                }
                Err(error) => tokenizer.crash(error),
            }
        }
        Some(_) => {
            tokenizer.enter(names.chunk);
            State::Retry(inside)
        }
    }
}

fn inside(tokenizer: &mut Tokenizer) -> State {
    let chunk = tokenizer.tokenize_state.mdx.expression.chunk;
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(chunk);
            State::Retry(before)
        }
        Some(b'}') if tokenizer.tokenize_state.mdx.size == 0 => {
            tokenizer.exit(chunk);
            State::Retry(before)
        }
        Some(byte) => {
            let agnostic = tokenizer
                .parse_state
                .mdx
                .as_ref()
                .is_none_or(|mdx| mdx.0.is_none());
            let state = &mut tokenizer.tokenize_state.mdx;
            if byte == b'{' && agnostic {
                state.size += 1;
            } else if byte == b'}' {
                state.size -= 1;
            }
            tokenizer.consume();
            State::Next(inside)
        }
    }
}

fn eol_after(tokenizer: &mut Tokenizer) -> State {
    let state = &tokenizer.tokenize_state.mdx;
    if tokenizer.point.line != state.line && !state.allow_lazy && tokenizer.lazy {
        return tokenizer.crash(Error::new(
            format!("{SOURCE}:unexpected-lazy"),
            "Unexpected lazy line in expression in container, expected line to be prefixed with `>` when in a block quote, whitespace when in a list, etc",
        ));
    }
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.enter(LINE_PREFIX);
        tokenizer.tokenize_state.mdx.prefix = 0;
        return State::Retry(prefix);
    }
    State::Retry(before)
}

fn prefix(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' '))
        && tokenizer.tokenize_state.mdx.prefix < INDENT_SIZE
    {
        tokenizer.tokenize_state.mdx.prefix += 1;
        tokenizer.consume();
        return State::Next(prefix);
    }
    tokenizer.exit(LINE_PREFIX);
    State::Retry(before)
}

// Whether the value so far ends the expression at the current `}`; a failure to parse is
// remembered, to be thrown if no later `}` ends it, as micromark does with acorn's errors
fn try_end(tokenizer: &mut Tokenizer) -> Result<bool, Error> {
    let Some(parser) = tokenizer
        .parse_state
        .mdx
        .as_ref()
        .and_then(|mdx| mdx.0.clone())
    else {
        return Ok(true);
    };
    let chunk = tokenizer.tokenize_state.mdx.expression.chunk;
    let state = &mut tokenizer.tokenize_state.mdx;
    state.events_read = append_script_source(
        &tokenizer.events,
        tokenizer.parse_state.bytes,
        state.events_read,
        chunk,
        &mut state.source,
    );
    let state = &mut tokenizer.tokenize_state.mdx;
    // A value that failed before its end has content no later text turns into a comment
    if state.hard_crash {
        return Ok(false);
    }
    if is_empty_script(&state.source) {
        if !state.allow_empty {
            return Err(Error::new(
                format!("{SOURCE}:unexpected-empty-expression"),
                "Unexpected empty expression",
            ));
        }
        return Ok(true);
    }
    let (prefix, suffix) = if state.spread { ("({", "})") } else { ("", "") };
    let value = format!("{prefix}{}{suffix}", state.source);
    let source_end = prefix.len() + state.source.len();
    if !state.spend_parse(value.len()) {
        state.hard_crash = true;
        state.crash.get_or_insert_with(acorn_error);
        return Ok(false);
    }
    let parsed = parser.parse_expression(&value);
    match parsed {
        Ok(expression)
            if is_empty_script(&value[expression.end.min(source_end)..source_end]) =>
        {
            if state.spread {
                check_spread(expression.object)?;
            }
            Ok(true)
        }
        // Content after the expression: a later `}` may put it in a comment
        Ok(_) => {
            state.crash = Some(acorn_error());
            Ok(false)
        }
        Err(error) => {
            state.hard_crash = error.end < source_end;
            state.crash = Some(acorn_error());
            Ok(false)
        }
    }
}

fn acorn_error() -> Error {
    Error::new(
        format!("{SOURCE}:acorn"),
        "Could not parse expression with acorn",
    )
}

fn check_spread(object: Option<Vec<(String, usize)>>) -> Result<(), Error> {
    let Some(properties) = object else {
        return Err(Error::new(
            format!("{SOURCE}:non-spread"),
            "Unexpected `ExpressionStatement` in code: expected an object spread (`{...spread}`)",
        ));
    };
    if properties.len() > 1 {
        return Err(Error::new(
            format!("{SOURCE}:spread-extra"),
            "Unexpected extra content in spread: only a single spread is supported",
        ));
    }
    match properties.first() {
        Some((kind, _)) if kind != "SpreadElement" => Err(Error::new(
            format!("{SOURCE}:non-spread"),
            format!("Unexpected `{kind}` in code: only spread elements are supported"),
        )),
        _ => Ok(()),
    }
}
