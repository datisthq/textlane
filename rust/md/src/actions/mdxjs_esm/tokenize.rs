use textlane_core::Error;

use crate::actions::blank_line::tokenize::tokenize_blank_line;
use crate::helpers::script::append_script_source;
use crate::models::construct::State;
use crate::models::mdx::{MDXJS_ESM, MDXJS_ESM_DATA};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const SOURCE: &str = "micromark-extension-mdxjs-esm";
const ALLOWED: [&str; 4] = [
    "ExportAllDeclaration",
    "ExportDefaultDeclaration",
    "ExportNamedDeclaration",
    "ImportDeclaration",
];

/// ESM, `import a from 'b'` or `export …` at the start of a line, as
/// micromark-extension-mdxjs-esm: it runs to a blank line where its JavaScript parses, and
/// may hold only imports and exports. Without a script parser any blank line ends it.
pub(crate) fn tokenize_mdxjs_esm(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.interrupt || tokenizer.point.column > 1 {
        return State::Nok;
    }
    tokenizer.enter(MDXJS_ESM);
    let events = tokenizer.events.len();
    let state = &mut tokenizer.tokenize_state.mdx;
    state.events_start = events;
    state.events_read = events;
    state.source.clear();
    state.size = 0;
    state.parsed = 0;
    tokenizer.enter(MDXJS_ESM_DATA);
    State::Retry(word)
}

fn word(tokenizer: &mut Tokenizer) -> State {
    let start = tokenizer.tokenize_state.mdx.events_start;
    let begin = tokenizer.events[start].point.index;
    let buffer = &tokenizer.parse_state.bytes[begin..tokenizer.point.index];
    match tokenizer.current {
        Some(byte) if byte.is_ascii_alphabetic() => {
            tokenizer.consume();
            State::Next(word)
        }
        Some(b' ') if buffer == b"import" || buffer == b"export" => {
            tokenizer.concrete = true;
            tokenizer.consume();
            State::Next(inside)
        }
        _ => State::Nok,
    }
}

fn inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(MDXJS_ESM_DATA);
            State::Retry(line_start)
        }
        Some(_) => {
            tokenizer.consume();
            State::Next(inside)
        }
    }
}

fn line_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => State::Retry(at_end),
        Some(b'\n') => {
            tokenizer.check(State::Next(at_end), State::Next(continuation_start));
            State::Retry(blank_line_before)
        }
        Some(_) => {
            tokenizer.enter(MDXJS_ESM_DATA);
            State::Retry(inside)
        }
    }
}

fn blank_line_before(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::BLANK_LINE_ENDING);
    tokenizer.consume();
    tokenizer.exit(Name::BLANK_LINE_ENDING);
    State::Next(tokenize_blank_line)
}

fn continuation_start(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::LINE_ENDING);
    tokenizer.consume();
    tokenizer.exit(Name::LINE_ENDING);
    State::Next(line_start)
}

fn at_end(tokenizer: &mut Tokenizer) -> State {
    match check(tokenizer) {
        Ok(true) => {
            tokenizer.exit(MDXJS_ESM);
            tokenizer.concrete = false;
            State::Ok
        }
        Ok(false) => State::Retry(continuation_start),
        Err(error) => tokenizer.crash(error),
    }
}

// Whether the ESM ends here: its JavaScript parses, or fails at its very end where more
// lines may finish it, which needs a line to follow
fn check(tokenizer: &mut Tokenizer) -> Result<bool, Error> {
    let Some(parser) = tokenizer
        .parse_state
        .mdx
        .as_ref()
        .and_then(|mdx| mdx.0.clone())
    else {
        return Ok(true);
    };
    let state = &mut tokenizer.tokenize_state.mdx;
    state.events_read = append_script_source(
        &tokenizer.events,
        tokenizer.parse_state.bytes,
        state.events_read,
        MDXJS_ESM_DATA,
        &mut state.source,
    );
    let at_eof = tokenizer.current.is_none();
    let state = &mut tokenizer.tokenize_state.mdx;
    // Imports declared before are declared again here, so redeclaring one fails
    let prefix = if state.imports.is_empty() {
        String::new()
    } else {
        format!("var {}\n", state.imports.join(","))
    };
    let value = format!("{prefix}{}", state.source);
    if !state.spend_parse(value.len()) {
        return Err(Error::new(
            format!("{SOURCE}:acorn"),
            "Could not parse import/exports with acorn",
        ));
    }
    let statements = match parser.parse_module(&value) {
        Ok(statements) => statements,
        Err(error) => {
            if !at_eof && error.end >= value.len() {
                return Ok(false);
            }
            return Err(Error::new(
                format!("{SOURCE}:acorn"),
                "Could not parse import/exports with acorn",
            ));
        }
    };
    let skip = usize::from(!prefix.is_empty());
    let mut imports = vec![];
    for statement in statements.iter().skip(skip) {
        if !ALLOWED.contains(&statement.kind.as_str()) {
            return Err(Error::new(
                format!("{SOURCE}:non-esm"),
                format!(
                    "Unexpected `{}` in code: only import/exports are supported",
                    statement.kind
                ),
            ));
        }
        imports.extend(statement.imports.iter().cloned());
    }
    state.imports.extend(imports);
    Ok(true)
}
