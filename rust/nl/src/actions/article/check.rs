use textlane_core::Error;
use textlane_core::nlcst::Node;

use crate::actions::nlcst::stringify::stringify_nlcst;
use crate::helpers::article_table::{
    A_INSENSITIVE, A_PREFIXES, A_SENSITIVE, AN_INSENSITIVE, AN_PREFIXES, AN_SENSITIVE,
};
use crate::helpers::number::{MAX_SAFE_INTEGER, read_first_number_word};
use crate::helpers::words::{WordPlace, visit_words};
use crate::models::report::Report;

/// Report `a` before a vowel sound and `an` before a consonant sound, as
/// retext-indefinite-article does.
pub(crate) fn check_articles<'a>(
    tree: &Node<'a>,
    reports: &mut Vec<Report<'a>>,
) -> Result<(), Error> {
    let mut failure = None;
    visit_words(tree, |node, place| {
        if failure.is_some() {
            return;
        }
        match check_article(node, place) {
            Ok(Some(report)) => reports.push(report),
            Ok(None) => {}
            Err(error) => failure = Some(error),
        }
    });
    failure.map_or(Ok(()), Err)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Article {
    A,
    An,
    Either,
}

fn check_article<'a>(
    node: &Node<'a>,
    place: &WordPlace<'_, 'a>,
) -> Result<Option<Report<'a>>, Error> {
    let actual = stringify_nlcst(node);
    let normal = actual.to_lowercase();
    if normal != "a" && normal != "an" {
        return Ok(None);
    }
    let Some(next) = find_next_word(place) else {
        return Ok(None);
    };
    let an = actual.encode_utf16().count() != 1;
    let following = stringify_nlcst(next);
    // `Station A equals`: a capital `A` inside a sentence is a letter
    if normal != actual && !an && place.first_word < place.index {
        return Ok(None);
    }
    // `a and b`: a lower-case `a` used as a letter
    if normal == actual
        && !an
        && ["and", "nor", "or"]
            .iter()
            .any(|word| following.eq_ignore_ascii_case(word))
    {
        return Ok(None);
    }
    let expected = match classify_word(&following)? {
        Some(Article::An) if !an => "an",
        Some(Article::A) if an => "a",
        _ => return Ok(None),
    };
    let expected = match (normal != actual, expected) {
        (true, "an") => "An",
        (true, _) => "A",
        (false, expected) => expected,
    };
    Ok(Some(Report {
        reason: format!(
            "Unexpected article `{actual}` before `{following}`, expected `{expected}`"
        ),
        rule: "retext-indefinite-article".to_string(),
        source: "retext-indefinite-article",
        position: node.position().copied(),
    }))
}

// The word after white space, and after one quote or bracket there
fn find_next_word<'n, 'a>(place: &WordPlace<'n, 'a>) -> Option<&'n Node<'a>> {
    let siblings = place.siblings;
    let mut index = place.index + 1;
    if !matches!(siblings.get(index)?, Node::WhiteSpace(_)) {
        return None;
    }
    index += 1;
    if let Some(sibling @ Node::Punctuation(_)) = siblings.get(index)
        && matches!(
            stringify_nlcst(sibling).as_str(),
            "“" | "”" | "‘" | "’" | "'" | "\"" | "(" | ")" | "[" | "]"
        )
    {
        index += 1;
    }
    siblings
        .get(index)
        .filter(|sibling| matches!(sibling, Node::Word(_)))
}

fn classify_word(value: &str) -> Result<Option<Article>, Error> {
    let digits = value.bytes().take_while(u8::is_ascii_digit).count();
    let head = if digits > 0 {
        read_first_number_word(read_number(&value[..digits])?)
    } else {
        value
            .split(['\'', '’', ' ', '-'])
            .next()
            .unwrap_or_default()
    };
    let a = matches_list(head, &A_SENSITIVE, &A_INSENSITIVE, &A_PREFIXES);
    let an = matches_list(head, &AN_SENSITIVE, &AN_INSENSITIVE, &AN_PREFIXES);
    Ok(match (a, an) {
        (true, true) => Some(Article::Either),
        (true, false) => Some(Article::A),
        (false, true) => Some(Article::An),
        _ if head.to_lowercase() == head => {
            if head.starts_with(['a', 'e', 'i', 'o', 'u']) {
                Some(Article::An)
            } else {
                Some(Article::A)
            }
        }
        _ => None,
    })
}

// number-to-words throws past `Number.MAX_SAFE_INTEGER`, which fails the whole run upstream
fn read_number(digits: &str) -> Result<u64, Error> {
    let mut number: u64 = 0;
    for byte in digits.bytes() {
        number = number * 10 + u64::from(byte - b'0');
        if number > MAX_SAFE_INTEGER {
            return Err(Error::new(
                "unsafe-number",
                "Input is not a safe number, it’s either too large or too small.",
            ));
        }
    }
    Ok(number)
}

fn matches_list(
    head: &str,
    sensitive: &[&str],
    insensitive: &[&str],
    prefixes: &[&str],
) -> bool {
    sensitive.binary_search(&head).is_ok()
        || insensitive
            .binary_search(&head.to_lowercase().as_str())
            .is_ok()
        || prefixes.iter().any(|prefix| {
            head.as_bytes()
                .get(..prefix.len())
                .is_some_and(|start| start.eq_ignore_ascii_case(prefix.as_bytes()))
        })
}
