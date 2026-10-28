use crate::actions::directive_name::tokenize::classify_directive_character;
use crate::actions::space_or_tab::tokenize::tokenize_space_or_tab;
use crate::actions::space_or_tab_eol::tokenize::tokenize_space_or_tab_eol;
use crate::helpers::character::CharacterKind;
use crate::models::construct::{State, StateFn};
use crate::models::directive::DirectiveNames;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// A directive's attributes, `{#a .b c=d e="f"}`; on one line unless the directive is a
/// text one.
pub(crate) fn tokenize_directive_attributes(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'{') {
        return State::Nok;
    }
    let names = read_names(tokenizer);
    tokenizer.enter(names.attributes);
    tokenizer.enter(names.attributes_marker);
    tokenizer.consume();
    tokenizer.exit(names.attributes_marker);
    State::Next(between)
}

fn read_names(tokenizer: &Tokenizer) -> &'static DirectiveNames {
    tokenizer.tokenize_state.directive.kind.names()
}

// Whitespace, with a line ending among it where the directive allows one
fn whitespace(tokenizer: &mut Tokenizer, next: StateFn) -> Option<State> {
    let disallow_eol = read_names(tokenizer).disallow_eol;
    match tokenizer.current {
        Some(b'\t' | b' ') if disallow_eol => {
            tokenizer.attempt(State::Next(next), State::Nok);
            Some(State::Retry(tokenize_space_or_tab(
                tokenizer,
                1,
                usize::MAX,
            )))
        }
        Some(b'\t' | b'\n' | b' ') if !disallow_eol => {
            tokenizer.attempt(State::Next(next), State::Nok);
            Some(State::Retry(tokenize_space_or_tab_eol(
                tokenizer, None, false,
            )))
        }
        _ => None,
    }
}

fn is_name_end(tokenizer: &Tokenizer, allowed: &[u8]) -> bool {
    match classify_directive_character(tokenizer) {
        CharacterKind::Whitespace => true,
        CharacterKind::Punctuation => {
            !matches!(tokenizer.current, Some(byte) if allowed.contains(&byte))
        }
        CharacterKind::Other => false,
    }
}

fn between(tokenizer: &mut Tokenizer) -> State {
    if let Some(byte @ (b'#' | b'.')) = tokenizer.current {
        tokenizer.tokenize_state.directive.class = byte == b'.';
        return State::Retry(shortcut_start);
    }
    if let Some(state) = whitespace(tokenizer, between) {
        return state;
    }
    if is_name_end(tokenizer, b"-_") {
        return State::Retry(end);
    }
    let names = read_names(tokenizer);
    tokenizer.enter(names.attribute);
    tokenizer.enter(names.attribute_name);
    tokenizer.consume();
    State::Next(name)
}

fn shortcut_names(tokenizer: &Tokenizer) -> (Name, Name, Name) {
    let names = read_names(tokenizer);
    if tokenizer.tokenize_state.directive.class {
        (
            names.attribute_class,
            names.attribute_class_marker,
            names.attribute_class_value,
        )
    } else {
        (
            names.attribute_id,
            names.attribute_id_marker,
            names.attribute_id_value,
        )
    }
}

fn shortcut_start(tokenizer: &mut Tokenizer) -> State {
    let (kind, marker, _) = shortcut_names(tokenizer);
    tokenizer.enter(read_names(tokenizer).attribute);
    tokenizer.enter(kind);
    tokenizer.enter(marker);
    tokenizer.consume();
    tokenizer.exit(marker);
    State::Next(shortcut_start_after)
}

fn shortcut_start_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None
        | Some(
            b'"' | b'#' | b'\'' | b'.' | b'<' | b'=' | b'>' | b'`' | b'}' | b'\t' | b'\n'
            | b' ',
        ) => State::Nok,
        Some(_) => {
            let (_, _, value) = shortcut_names(tokenizer);
            tokenizer.enter(value);
            tokenizer.consume();
            State::Next(shortcut)
        }
    }
}

fn shortcut(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'"' | b'\'' | b'<' | b'=' | b'>' | b'`') => State::Nok,
        Some(b'#' | b'.' | b'}' | b'\t' | b'\n' | b' ') => {
            let (kind, _, value) = shortcut_names(tokenizer);
            tokenizer.exit(value);
            tokenizer.exit(kind);
            tokenizer.exit(read_names(tokenizer).attribute);
            State::Retry(between)
        }
        Some(_) => {
            tokenizer.consume();
            State::Next(shortcut)
        }
    }
}

fn name(tokenizer: &mut Tokenizer) -> State {
    if !is_name_end(tokenizer, b"-.:_") {
        tokenizer.consume();
        return State::Next(name);
    }
    tokenizer.exit(read_names(tokenizer).attribute_name);
    if let Some(state) = whitespace(tokenizer, name_after) {
        return state;
    }
    State::Retry(name_after)
}

fn name_after(tokenizer: &mut Tokenizer) -> State {
    let names = read_names(tokenizer);
    if tokenizer.current == Some(b'=') {
        tokenizer.enter(names.attribute_initializer_marker);
        tokenizer.consume();
        tokenizer.exit(names.attribute_initializer_marker);
        return State::Next(value_before);
    }
    tokenizer.exit(names.attribute);
    State::Retry(between)
}

fn value_before(tokenizer: &mut Tokenizer) -> State {
    let names = read_names(tokenizer);
    match tokenizer.current {
        None | Some(b'<' | b'=' | b'>' | b'`' | b'}') => State::Nok,
        Some(b'\n') if names.disallow_eol => State::Nok,
        Some(marker @ (b'"' | b'\'')) => {
            tokenizer.enter(names.attribute_value_literal);
            tokenizer.enter(names.attribute_value_marker);
            tokenizer.consume();
            tokenizer.exit(names.attribute_value_marker);
            tokenizer.tokenize_state.directive.marker = marker;
            State::Next(value_quoted_start)
        }
        _ => {
            if let Some(state) = whitespace(tokenizer, value_before) {
                return state;
            }
            tokenizer.enter(names.attribute_value);
            tokenizer.enter(names.attribute_value_data);
            tokenizer.consume();
            State::Next(value_unquoted)
        }
    }
}

fn value_unquoted(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'"' | b'\'' | b'<' | b'=' | b'>' | b'`') => State::Nok,
        Some(b'}' | b'\t' | b'\n' | b' ') => {
            let names = read_names(tokenizer);
            tokenizer.exit(names.attribute_value_data);
            tokenizer.exit(names.attribute_value);
            tokenizer.exit(names.attribute);
            State::Retry(between)
        }
        Some(_) => {
            tokenizer.consume();
            State::Next(value_unquoted)
        }
    }
}

fn value_quoted_start(tokenizer: &mut Tokenizer) -> State {
    let names = read_names(tokenizer);
    if tokenizer.current == Some(tokenizer.tokenize_state.directive.marker) {
        tokenizer.enter(names.attribute_value_marker);
        tokenizer.consume();
        tokenizer.exit(names.attribute_value_marker);
        tokenizer.exit(names.attribute_value_literal);
        tokenizer.exit(names.attribute);
        return State::Next(value_quoted_after);
    }
    tokenizer.enter(names.attribute_value);
    State::Retry(value_quoted_between)
}

fn value_quoted_between(tokenizer: &mut Tokenizer) -> State {
    let names = read_names(tokenizer);
    match tokenizer.current {
        Some(byte) if byte == tokenizer.tokenize_state.directive.marker => {
            tokenizer.exit(names.attribute_value);
            State::Retry(value_quoted_start)
        }
        None => State::Nok,
        Some(b'\n') => {
            if names.disallow_eol {
                return State::Nok;
            }
            tokenizer.attempt(State::Next(value_quoted_between), State::Nok);
            State::Retry(tokenize_space_or_tab_eol(tokenizer, None, false))
        }
        Some(_) => {
            tokenizer.enter(names.attribute_value_data);
            tokenizer.consume();
            State::Next(value_quoted)
        }
    }
}

fn value_quoted(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(byte)
            if byte != tokenizer.tokenize_state.directive.marker && byte != b'\n' =>
        {
            tokenizer.consume();
            State::Next(value_quoted)
        }
        _ => {
            tokenizer.exit(read_names(tokenizer).attribute_value_data);
            State::Retry(value_quoted_between)
        }
    }
}

fn value_quoted_after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'}' | b'\t' | b'\n' | b' ') => State::Retry(between),
        _ => State::Retry(end),
    }
}

fn end(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'}') {
        return State::Nok;
    }
    let names = read_names(tokenizer);
    tokenizer.enter(names.attributes_marker);
    tokenizer.consume();
    tokenizer.exit(names.attributes_marker);
    tokenizer.exit(names.attributes);
    State::Ok
}
