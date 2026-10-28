use crate::models::construct::State;
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

const SCHEME_SIZE_MAX: usize = 32;
const DOMAIN_SIZE_MAX: usize = 63;

/// An autolink: `<https://example.com>` or `<a@example.com>`.
pub(crate) fn tokenize_autolink(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current != Some(b'<') {
        return State::Nok;
    }
    tokenizer.enter(Name::AUTOLINK);
    tokenizer.enter(Name::AUTOLINK_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::AUTOLINK_MARKER);
    tokenizer.enter(Name::AUTOLINK_PROTOCOL);
    State::Next(open)
}

fn open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.consume();
            State::Next(scheme_or_email_atext)
        }
        Some(b'@') => State::Nok,
        _ => State::Retry(email_atext),
    }
}

fn scheme_or_email_atext(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'+' | b'-' | b'.' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z') => {
            tokenizer.tokenize_state.size = 1;
            State::Retry(scheme_inside_or_email_atext)
        }
        _ => State::Retry(email_atext),
    }
}

fn scheme_inside_or_email_atext(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b':') => {
            tokenizer.consume();
            tokenizer.tokenize_state.size = 0;
            State::Next(url_inside)
        }
        Some(b'+' | b'-' | b'.' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z')
            if tokenizer.tokenize_state.size < SCHEME_SIZE_MAX =>
        {
            tokenizer.consume();
            tokenizer.tokenize_state.size += 1;
            State::Next(scheme_inside_or_email_atext)
        }
        _ => {
            tokenizer.tokenize_state.size = 0;
            State::Retry(email_atext)
        }
    }
}

fn url_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'>') => {
            tokenizer.exit(Name::AUTOLINK_PROTOCOL);
            close(tokenizer)
        }
        None | Some(b'\0'..=0x1F | b' ' | b'<' | 0x7F) => State::Nok,
        Some(_) => {
            tokenizer.consume();
            State::Next(url_inside)
        }
    }
}

fn close(tokenizer: &mut Tokenizer) -> State {
    tokenizer.enter(Name::AUTOLINK_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::AUTOLINK_MARKER);
    tokenizer.exit(Name::AUTOLINK);
    State::Ok
}

fn email_atext(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'@') => {
            tokenizer.consume();
            State::Next(email_at_sign_or_dot)
        }
        Some(
            b'#'..=b'\''
            | b'*'
            | b'+'
            | b'-'..=b'9'
            | b'='
            | b'?'
            | b'A'..=b'Z'
            | b'^'..=b'~',
        ) => {
            tokenizer.consume();
            State::Next(email_atext)
        }
        _ => State::Nok,
    }
}

fn email_at_sign_or_dot(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z') => State::Retry(email_value),
        _ => State::Nok,
    }
}

fn email_label(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'.') => {
            tokenizer.consume();
            tokenizer.tokenize_state.size = 0;
            State::Next(email_at_sign_or_dot)
        }
        Some(b'>') => {
            // What looked like a protocol is an email address
            let index = tokenizer.events.len();
            tokenizer.exit(Name::AUTOLINK_PROTOCOL);
            tokenizer.events[index - 1].name = Name::AUTOLINK_EMAIL;
            tokenizer.events[index].name = Name::AUTOLINK_EMAIL;
            tokenizer.tokenize_state.size = 0;
            close(tokenizer)
        }
        _ => State::Retry(email_value),
    }
}

fn email_value(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(byte @ (b'-' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z'))
            if tokenizer.tokenize_state.size < DOMAIN_SIZE_MAX =>
        {
            tokenizer.tokenize_state.size += 1;
            tokenizer.consume();
            State::Next(if byte == b'-' {
                email_value
            } else {
                email_label
            })
        }
        _ => {
            tokenizer.tokenize_state.size = 0;
            State::Nok
        }
    }
}
