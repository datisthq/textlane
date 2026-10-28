use textlane_core::mdast::{Link, Node, Text};
use textlane_core::{Allocator, ArenaVec};

use crate::helpers::character::{
    CharacterKind, classify_character_unit, find_character_before,
};

/// Link the URLs and email addresses micromark left as text, as
/// mdast-util-gfm-autolink-literal's find-and-replace does after parsing: URLs first, then
/// emails, in text outside links and link references. New nodes have no position.
pub(crate) fn autolink_text<'a>(root: &mut Node<'a>, alloc: &'a Allocator) {
    let mut stack = root.children_mut().into_iter().collect::<Vec<_>>();
    while let Some(children) = stack.pop() {
        replace_children(children, alloc);
        for child in children.iter_mut() {
            if !matches!(child, Node::Link(_) | Node::LinkReference(_))
                && let Some(children) = child.children_mut()
            {
                stack.push(children);
            }
        }
    }
}

// A piece of a text node's value, by byte range: text, or a link with that text
#[derive(Debug)]
enum Piece {
    Text(usize, usize),
    Link(usize, usize, Prefix),
}

#[derive(Clone, Copy, Debug)]
enum Prefix {
    None,
    Http,
    Mailto,
}

fn replace_children<'a>(children: &mut ArenaVec<'a, Node<'a>>, alloc: &'a Allocator) {
    let replacements = children
        .iter()
        .map(|child| match child {
            Node::Text(text) => find_pieces(text.value),
            _ => None,
        })
        .collect::<Vec<_>>();
    if replacements.iter().all(Option::is_none) {
        return;
    }
    let old = std::mem::replace(children, ArenaVec::new_in(alloc));
    for (child, pieces) in old.into_iter().zip(replacements) {
        match (child, pieces) {
            (Node::Text(text), Some(pieces)) => {
                for piece in pieces {
                    children.push(create_node(text.value, piece, alloc));
                }
            }
            (child, _) => children.push(child),
        }
    }
}

fn create_node<'a>(value: &'a str, piece: Piece, alloc: &'a Allocator) -> Node<'a> {
    let text = |start, end| {
        Node::Text(Text {
            value: &value[start..end],
            position: None,
        })
    };
    match piece {
        Piece::Text(start, end) => text(start, end),
        Piece::Link(start, end, prefix) => {
            let raw = &value[start..end];
            let url = match prefix {
                Prefix::None => raw,
                Prefix::Http => alloc.alloc_str(&format!("http://{raw}")),
                Prefix::Mailto => alloc.alloc_str(&format!("mailto:{raw}")),
            };
            let mut children = ArenaVec::with_capacity_in(1, alloc);
            children.push(text(start, end));
            Node::Link(Link {
                url,
                title: None,
                children,
                position: None,
            })
        }
    }
}

// The URL pass, then the email pass over each text piece it leaves; `None` when neither
// links anything
fn find_pieces(value: &str) -> Option<Vec<Piece>> {
    let urls = find_urls(value);
    let mut pieces = vec![];
    let mut changed = urls.is_some();
    for piece in urls.unwrap_or_else(|| vec![Piece::Text(0, value.len())]) {
        match piece {
            Piece::Text(start, end) => match find_emails(value, start, end) {
                Some(emails) => {
                    changed = true;
                    pieces.extend(emails);
                }
                None => push_piece(&mut pieces, piece),
            },
            piece => pieces.push(piece),
        }
    }
    changed.then_some(pieces)
}

// Adjacent text joins, as find-and-replace's `add` does
fn push_piece(pieces: &mut Vec<Piece>, piece: Piece) {
    if let (Some(Piece::Text(_, last_end)), Piece::Text(start, end)) =
        (pieces.last_mut(), &piece)
        && *last_end == *start
    {
        *last_end = *end;
        return;
    }
    pieces.push(piece);
}

// Matches found with `lastIndex` semantics: a rejected match retries one character on
fn collect_matches(
    value: &str,
    start: usize,
    end: usize,
    mut find: impl FnMut(usize) -> Option<(Vec<Piece>, usize)>,
) -> Option<Vec<Piece>> {
    let mut pieces = vec![];
    let mut copied = start;
    let mut index = start;
    while index < end {
        if !value.is_char_boundary(index) {
            index += 1;
            continue;
        }
        match find(index) {
            Some((found, after)) => {
                if copied < index {
                    push_piece(&mut pieces, Piece::Text(copied, index));
                }
                for piece in found {
                    push_piece(&mut pieces, piece);
                }
                copied = after;
                index = after;
            }
            None => index += 1,
        }
    }
    if pieces.is_empty() {
        return None;
    }
    if copied < end {
        push_piece(&mut pieces, Piece::Text(copied, end));
    }
    Some(pieces)
}

// Upstream's `previous`: the start of the text node, which a piece the URL pass left is,
// or after whitespace or punctuation, read as a UTF-16 code unit
fn is_previous_allowed(value: &str, start: usize, index: usize, email: bool) -> bool {
    if index == start {
        return true;
    }
    let character = find_character_before(value.as_bytes(), index);
    classify_character_unit(character) != CharacterKind::Other
        && !(email && character == Some('/'))
}

fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// What a run of domain characters ends in, shared by every URL that starts in it: the
/// verdict on its last two segments holds for each that starts before the second-last dot.
#[derive(Default)]
struct DomainRun {
    start: usize,
    end: usize,
    second_last_dot: Option<usize>,
    correct: bool,
}

// `/(https?:\/\/|www(?=\.))([-.\w]+)([^ \t\r\n]*)/gi`, then `findUrl`
fn find_urls(value: &str) -> Option<Vec<Piece>> {
    let bytes = value.as_bytes();
    let mut run = DomainRun::default();
    collect_matches(value, 0, value.len(), |index| {
        let rest = &bytes[index..];
        let starts_with = |prefix: &[u8]| {
            rest.len() >= prefix.len()
                && rest[..prefix.len()].eq_ignore_ascii_case(prefix)
        };
        let (www, domain_start) = if starts_with(b"www.") {
            (true, index + 3)
        } else if starts_with(b"https://") {
            (false, index + 8)
        } else if starts_with(b"http://") {
            (false, index + 7)
        } else {
            return None;
        };
        if !(domain_start < bytes.len() && is_domain(bytes[domain_start]))
            || !is_previous_allowed(value, 0, index, false)
        {
            return None;
        }
        if !(run.start <= domain_start && domain_start < run.end) {
            run = read_domain_run(bytes, domain_start);
        }
        // `www` is part of the domain
        let domain = if www { index } else { domain_start };
        let correct = match run.second_last_dot {
            Some(dot) if domain <= dot => run.correct,
            _ => is_correct_domain(&value[domain..run.end]),
        };
        if !correct {
            return None;
        }
        let end = run.end
            + bytes[run.end..]
                .iter()
                .take_while(|byte| !matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
                .count();
        let url_end = split_url(bytes, domain, end);
        if url_end == domain {
            return None;
        }
        let prefix = if www { Prefix::Http } else { Prefix::None };
        let mut found = vec![Piece::Link(index, url_end, prefix)];
        if url_end < end {
            found.push(Piece::Text(url_end, end));
        }
        Some((found, end))
    })
}

fn is_domain(byte: u8) -> bool {
    is_word(byte) || matches!(byte, b'-' | b'.')
}

fn read_domain_run(bytes: &[u8], start: usize) -> DomainRun {
    let end = start
        + bytes[start..]
            .iter()
            .take_while(|byte| is_domain(**byte))
            .count();
    let mut dots = bytes[start..end]
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, byte)| **byte == b'.')
        .map(|(offset, _)| start + offset);
    let _last = dots.next();
    let second_last_dot = dots.next();
    let correct = second_last_dot.is_some_and(|dot| {
        std::str::from_utf8(&bytes[dot..end]).is_ok_and(is_correct_domain)
    });
    DomainRun {
        start,
        end,
        second_last_dot,
        correct,
    }
}

// At least two segments, and neither of the last two has `_` or lacks an alphanumeric
fn is_correct_domain(domain: &str) -> bool {
    let parts = domain.split('.').collect::<Vec<_>>();
    parts.len() >= 2
        && parts[parts.len() - 2..].iter().all(|part| {
            part.is_empty()
                || (!part.contains('_')
                    && part.bytes().any(|byte| byte.is_ascii_alphanumeric()))
        })
}

// The end of the URL in `start..end` without its trailing punctuation, but with each
// closing paren the URL has an opening one for
fn split_url(bytes: &[u8], start: usize, end: usize) -> usize {
    let is_trail = |byte: &u8| {
        matches!(
            byte,
            b'!' | b'"'
                | b'&'
                | b'\''
                | b')'
                | b','
                | b'.'
                | b':'
                | b';'
                | b'<'
                | b'>'
                | b'?'
                | b']'
                | b'}'
        )
    };
    let trail = bytes[start..end]
        .iter()
        .rev()
        .take_while(|byte| is_trail(byte))
        .count();
    let mut url_end = end - trail;
    let url = &bytes[start..url_end];
    let opening = url.iter().filter(|byte| **byte == b'(').count();
    let mut closing = url.iter().filter(|byte| **byte == b')').count();
    while opening > closing {
        let Some(offset) = bytes[url_end..end].iter().position(|byte| *byte == b')')
        else {
            break;
        };
        url_end += offset + 1;
        closing += 1;
    }
    url_end
}

/// The atext run an email may start in, and the label after its `@`, shared by every
/// start in the run.
#[derive(Default)]
struct AtextRun {
    start: usize,
    end: usize,
    label_end: Option<usize>,
}

// `/(?<=^|\s|\p{P}|\p{S})([-.\w+]+)@([-\w]+(?:\.[-\w]+)+)/gu`, then `findEmail`
fn find_emails(value: &str, start: usize, end: usize) -> Option<Vec<Piece>> {
    let bytes = &value.as_bytes()[..end];
    let mut run = AtextRun::default();
    collect_matches(value, start, end, |index| {
        let is_atext = |byte: u8| is_word(byte) || matches!(byte, b'-' | b'.' | b'+');
        if !is_atext(bytes[index]) || !is_previous_allowed(value, start, index, true) {
            return None;
        }
        if !(run.start <= index && index < run.end) {
            run = read_atext_run(bytes, index, is_atext);
        }
        let label_end = run.label_end?;
        Some((
            vec![Piece::Link(index, label_end, Prefix::Mailto)],
            label_end,
        ))
    })
}

fn read_atext_run(bytes: &[u8], start: usize, is_atext: impl Fn(u8) -> bool) -> AtextRun {
    let end = start
        + bytes[start..]
            .iter()
            .take_while(|byte| is_atext(**byte))
            .count();
    let label_end = (bytes.get(end) == Some(&b'@'))
        .then(|| read_label_end(bytes, end + 1))
        .flatten()
        // A label may not end in `-`, a digit or `_`
        .filter(|label_end| {
            let last = bytes[label_end - 1];
            !(last == b'-' || last == b'_' || last.is_ascii_digit())
        });
    AtextRun {
        start,
        end,
        label_end,
    }
}

// `[-\w]+(?:\.[-\w]+)+`
fn read_label_end(bytes: &[u8], start: usize) -> Option<usize> {
    let is_label = |byte: &u8| is_word(*byte) || *byte == b'-';
    let mut end = start
        + bytes[start..]
            .iter()
            .take_while(|byte| is_label(byte))
            .count();
    if end == start {
        return None;
    }
    let mut segments = 0;
    while bytes.get(end) == Some(&b'.') {
        let size = bytes[end + 1..]
            .iter()
            .take_while(|byte| is_label(byte))
            .count();
        if size == 0 {
            break;
        }
        end += 1 + size;
        segments += 1;
    }
    (segments > 0).then_some(end)
}
