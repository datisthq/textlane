use crate::helpers::character::CharacterKind;
use crate::helpers::literal::{
    can_start_email, can_start_protocol, can_start_www, classify_unit_at, is_atext,
};
use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// A GFM literal autolink: an email, then a `www.` or an `http(s)://` literal, tried in
/// the order micromark tries them.
pub(crate) fn tokenize_gfm_autolink_literal(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Ok, State::Next(before_url));
    State::Retry(email_start)
}

fn before_url(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'W' | b'w') => State::Retry(www_start),
        Some(b'H' | b'h') => State::Retry(protocol_start),
        _ => State::Nok,
    }
}

// Not inside a link label that may still close: the link wins over the literal
fn is_allowed(tokenizer: &Tokenizer) -> bool {
    tokenizer.tokenize_state.label_starts.is_empty()
}

fn email_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(byte)
            if can_start_email(tokenizer.previous, byte) && is_allowed(tokenizer) =>
        {
            tokenizer.enter(Name::LITERAL_AUTOLINK);
            tokenizer.enter(Name::LITERAL_AUTOLINK_EMAIL);
            State::Retry(email_atext)
        }
        _ => State::Nok,
    }
}

fn email_atext(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'@') => {
            tokenizer.consume();
            State::Next(email_domain)
        }
        Some(byte) if is_atext(byte) => {
            tokenizer.consume();
            State::Next(email_atext)
        }
        _ => State::Nok,
    }
}

fn email_domain(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'.') => {
            tokenizer.check(
                State::Next(email_domain_after),
                State::Next(email_domain_dot),
            );
            State::Retry(email_dot_trail)
        }
        Some(byte) if byte == b'-' || byte == b'_' || byte.is_ascii_alphanumeric() => {
            tokenizer.tokenize_state.seen = true;
            tokenizer.consume();
            State::Next(email_domain)
        }
        _ => State::Retry(email_domain_after),
    }
}

fn email_dot_trail(tokenizer: &mut Tokenizer) -> State {
    tokenizer.consume();
    State::Next(email_dot_trail_after)
}

fn email_dot_trail_after(tokenizer: &mut Tokenizer) -> State {
    if tokenizer
        .current
        .is_some_and(|byte| byte.is_ascii_alphanumeric())
    {
        State::Nok
    } else {
        State::Ok
    }
}

fn email_domain_dot(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.marker = b'.';
    tokenizer.consume();
    State::Next(email_domain)
}

// The domain is not empty, has a dot and ends in a letter
fn email_domain_after(tokenizer: &mut Tokenizer) -> State {
    let valid = tokenizer.tokenize_state.seen
        && tokenizer.tokenize_state.marker == b'.'
        && tokenizer
            .previous
            .is_some_and(|byte| byte.is_ascii_alphabetic());
    tokenizer.tokenize_state.seen = false;
    tokenizer.tokenize_state.marker = 0;
    if !valid {
        return State::Nok;
    }
    tokenizer.exit(Name::LITERAL_AUTOLINK_EMAIL);
    tokenizer.exit(Name::LITERAL_AUTOLINK);
    State::Ok
}

fn www_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(byte)
            if can_start_www(tokenizer.previous, byte) && is_allowed(tokenizer) =>
        {
            tokenizer.enter(Name::LITERAL_AUTOLINK);
            tokenizer.enter(Name::LITERAL_AUTOLINK_WWW);
            // The prefix is only checked: it is part of the domain
            tokenizer.check(State::Next(www_domain), State::Nok);
            State::Retry(www_prefix_inside)
        }
        _ => State::Nok,
    }
}

fn www_prefix_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'W' | b'w') if tokenizer.tokenize_state.size < 3 => {
            tokenizer.tokenize_state.size += 1;
            tokenizer.consume();
            State::Next(www_prefix_inside)
        }
        Some(b'.') if tokenizer.tokenize_state.size == 3 => {
            tokenizer.tokenize_state.size = 0;
            tokenizer.consume();
            State::Next(www_prefix_after)
        }
        _ => {
            tokenizer.tokenize_state.size = 0;
            State::Nok
        }
    }
}

fn www_prefix_after(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current.is_none() {
        State::Nok
    } else {
        State::Ok
    }
}

fn www_domain(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(www_path), State::Nok);
    State::Retry(domain_start)
}

fn www_path(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(www_after), State::Nok);
    State::Retry(path_inside)
}

fn www_after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(Name::LITERAL_AUTOLINK_WWW);
    tokenizer.exit(Name::LITERAL_AUTOLINK);
    State::Ok
}

fn protocol_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(byte)
            if can_start_protocol(tokenizer.previous, byte) && is_allowed(tokenizer) =>
        {
            tokenizer.enter(Name::LITERAL_AUTOLINK);
            tokenizer.enter(Name::LITERAL_AUTOLINK_HTTP);
            tokenizer.tokenize_state.start = tokenizer.point.index as usize;
            tokenizer.consume();
            State::Next(protocol_prefix_inside)
        }
        _ => State::Nok,
    }
}

fn protocol_prefix_inside(tokenizer: &mut Tokenizer) -> State {
    let start = tokenizer.tokenize_state.start;
    match tokenizer.current {
        // `5` is the size of `https`
        Some(byte)
            if byte.is_ascii_alphabetic()
                && (tokenizer.point.index as usize) - start < 5 =>
        {
            tokenizer.consume();
            State::Next(protocol_prefix_inside)
        }
        Some(b':') => {
            tokenizer.tokenize_state.start = 0;
            let name =
                &tokenizer.parse_state.bytes[start..tokenizer.point.index as usize];
            if name.eq_ignore_ascii_case(b"http") || name.eq_ignore_ascii_case(b"https") {
                tokenizer.consume();
                State::Next(protocol_slashes_inside)
            } else {
                State::Nok
            }
        }
        _ => {
            tokenizer.tokenize_state.start = 0;
            State::Nok
        }
    }
}

fn protocol_slashes_inside(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'/') {
        tokenizer.tokenize_state.size = 0;
        return State::Nok;
    }
    tokenizer.consume();
    if tokenizer.tokenize_state.size == 0 {
        tokenizer.tokenize_state.size = 1;
        State::Next(protocol_slashes_inside)
    } else {
        tokenizer.tokenize_state.size = 0;
        State::Next(protocol_after_slashes)
    }
}

// micromark's own check here, which markdown-rs lacks: the domain starts with neither
// whitespace, punctuation nor a control character
fn protocol_after_slashes(tokenizer: &mut Tokenizer) -> State {
    if tokenizer
        .current
        .is_none_or(|byte| byte.is_ascii_control() || is_space_or_punctuation(tokenizer))
    {
        return State::Nok;
    }
    tokenizer.attempt(State::Next(protocol_path), State::Nok);
    State::Retry(domain_start)
}

fn protocol_path(tokenizer: &mut Tokenizer) -> State {
    tokenizer.attempt(State::Next(protocol_after), State::Nok);
    State::Retry(path_inside)
}

fn protocol_after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.exit(Name::LITERAL_AUTOLINK_HTTP);
    tokenizer.exit(Name::LITERAL_AUTOLINK);
    State::Ok
}

fn is_space_or_punctuation(tokenizer: &Tokenizer) -> bool {
    let kind = classify_unit_at(
        tokenizer.parse_state.bytes,
        tokenizer.point.index as usize,
        tokenizer.current,
    );
    kind != CharacterKind::Other
}

fn is_whitespace(tokenizer: &Tokenizer) -> bool {
    let kind = classify_unit_at(
        tokenizer.parse_state.bytes,
        tokenizer.point.index as usize,
        tokenizer.current,
    );
    kind == CharacterKind::Whitespace
}

fn domain_start(tokenizer: &mut Tokenizer) -> State {
    let index = tokenizer.point.index;
    let (start, dot) = tokenizer.tokenize_state.literal_domain_failed;
    if start <= index as usize && index < dot as u32 {
        return State::Nok;
    }
    tokenizer.tokenize_state.literal_domain = (index as usize, 0, 0);
    State::Retry(domain_inside)
}

fn domain_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'.' | b'_') => {
            tokenizer.check(
                State::Next(domain_after),
                State::Next(domain_at_punctuation),
            );
            State::Retry(trail_start)
        }
        // The rest of a character whose first byte was classified
        Some(0x80..=0xBF) => {
            tokenizer.consume();
            State::Next(domain_inside)
        }
        Some(b'-') => {
            tokenizer.consume();
            State::Next(domain_inside)
        }
        _ if is_space_or_punctuation(tokenizer) => State::Retry(domain_after),
        _ => {
            tokenizer.tokenize_state.seen = true;
            tokenizer.consume();
            State::Next(domain_inside)
        }
    }
}

fn domain_at_punctuation(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'_') {
        tokenizer.tokenize_state.marker = b'_';
    } else {
        tokenizer.tokenize_state.marker_b = tokenizer.tokenize_state.marker;
        tokenizer.tokenize_state.marker = 0;
        let (start, last, _) = tokenizer.tokenize_state.literal_domain;
        tokenizer.tokenize_state.literal_domain =
            (start, tokenizer.point.index as usize, last);
    }
    tokenizer.consume();
    State::Next(domain_inside)
}

// No underscore in the last two segments, and something besides punctuation
fn domain_after(tokenizer: &mut Tokenizer) -> State {
    let state = &mut tokenizer.tokenize_state;
    let valid = state.marker_b != b'_' && state.marker != b'_' && state.seen;
    state.seen = false;
    state.marker = 0;
    state.marker_b = 0;
    if valid {
        return State::Ok;
    }
    let (start, _, second_last) = state.literal_domain;
    state.literal_domain_failed = (start, second_last);
    State::Nok
}

fn path_inside(tokenizer: &mut Tokenizer) -> State {
    let state = &tokenizer.tokenize_state;
    let paren_open = state.size_b < state.size;
    match tokenizer.current {
        Some(b'(') => {
            tokenizer.tokenize_state.size += 1;
            tokenizer.consume();
            State::Next(path_inside)
        }
        // A closing paren with an opening one left for it is never trailing
        Some(b')') if paren_open => State::Retry(path_at_punctuation),
        Some(
            b'!' | b'"' | b'&' | b'\'' | b')' | b'*' | b',' | b'.' | b':' | b';' | b'<'
            | b'?' | b']' | b'_' | b'~',
        ) => {
            tokenizer.check(State::Next(path_after), State::Next(path_at_punctuation));
            State::Retry(trail_start)
        }
        Some(0x80..=0xBF) => {
            tokenizer.consume();
            State::Next(path_inside)
        }
        _ if is_whitespace(tokenizer) => State::Retry(path_after),
        _ => {
            tokenizer.consume();
            State::Next(path_inside)
        }
    }
}

fn path_at_punctuation(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b')') {
        tokenizer.tokenize_state.size_b += 1;
    }
    tokenizer.consume();
    State::Next(path_inside)
}

fn path_after(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.size = 0;
    tokenizer.tokenize_state.size_b = 0;
    State::Ok
}

// Trailing punctuation, then the end of the URL: `ok` means none of it is part of the URL
fn trail_start(tokenizer: &mut Tokenizer) -> State {
    let index = tokenizer.point.index;
    let (start, failed) = tokenizer.tokenize_state.literal_trail;
    if start <= index as usize && index < failed as u32 {
        return State::Nok;
    }
    tokenizer.tokenize_state.literal_trail = (index as usize, 0);
    State::Retry(trail)
}

fn trail(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(
            b'!' | b'"' | b'\'' | b')' | b'*' | b',' | b'.' | b':' | b';' | b'?' | b'_'
            | b'~',
        ) => {
            tokenizer.consume();
            State::Next(trail)
        }
        // `&`, letters and `;` look like a character reference and trail as a whole
        Some(b'&') => {
            tokenizer.consume();
            State::Next(trail_character_reference_start)
        }
        // A `]` trails unless a resource or reference could follow it
        Some(b']') => {
            tokenizer.consume();
            State::Next(trail_bracket_after)
        }
        Some(b'<') => State::Ok,
        _ if is_whitespace(tokenizer) => State::Ok,
        _ => fail_trail(tokenizer),
    }
}

fn trail_bracket_after(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'(' | b'[')) || is_whitespace(tokenizer) {
        State::Ok
    } else {
        State::Retry(trail)
    }
}

fn trail_character_reference_start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer
        .current
        .is_some_and(|byte| byte.is_ascii_alphabetic())
    {
        State::Retry(trail_character_reference_inside)
    } else {
        fail_trail(tokenizer)
    }
}

fn trail_character_reference_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b';') => {
            tokenizer.consume();
            State::Next(trail)
        }
        Some(byte) if byte.is_ascii_alphabetic() => {
            tokenizer.consume();
            State::Next(trail_character_reference_inside)
        }
        _ => fail_trail(tokenizer),
    }
}

fn fail_trail(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.literal_trail.1 = tokenizer.point.index as usize;
    State::Nok
}
