use std::sync::LazyLock;

use regex::Regex;

use crate::models::highlight::{CodeBlock, CodeToken};

/// A comment Shiki's `parseComments` found at a line's end: its text cut into the comment
/// marker, the body and a closing marker, and where it is.
pub(crate) struct Comment {
    pub(crate) info: [String; 3],
    pub(crate) line: usize,
    pub(crate) token: usize,
    pub(crate) line_comment_only: bool,
    pub(crate) jsx_style: bool,
    pub(crate) additional: Vec<usize>,
}

static SPLIT_COMMENT: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"\s+//").ok());
static LINE_COMMENT_PREFIX: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r#"^(?://|["'#]|;{1,2}|%{1,2}|--)$"#).ok());
// Each with whether it must end the line, as Shiki lists them
static MATCHERS: LazyLock<Vec<(Regex, bool)>> = LazyLock::new(|| {
    [
        (r"^(<!--)(.+)(-->)$", false),
        (r"^(/\*)(.+)(\*/)$", false),
        (r#"^(//|["'#]|;{1,2}|%{1,2}|--)(.*)$"#, true),
        (r"^(\*)(.+)$", true),
    ]
    .into_iter()
    .filter_map(|(pattern, end)| Some((Regex::new(pattern).ok()?, end)))
    .collect()
});

/// Find the comments notations can sit in, as `parseComments` does with its `v3` matching:
/// tokens holding ` //` are split there first, then only the last token (the last two in
/// JSX) of each line is tried.
pub(crate) fn parse_comments(block: &mut CodeBlock, jsx: bool) -> Vec<Comment> {
    let mut comments = Vec::new();
    for line in block.visible_lines() {
        split_comments(&mut block.lines[line].tokens);
        let tokens = &block.lines[line].tokens;
        let length = tokens.len();
        // Shiki's numbers are tokens of the line, so a numbered line is never comment-only
        let numbered = usize::from(block.lines[line].number.is_some());
        let start = match jsx {
            true => length.saturating_sub(2),
            false => length.saturating_sub(1),
        };
        for index in start..length {
            let Some(token) = &tokens[index] else {
                continue;
            };
            let last = index == length - 1;
            let found = match_token(&token.text, last);
            // A notation tokenized apart from the comment it sits in joins it
            if found.is_none()
                && index > 0
                && token.text.trim().starts_with("[!code")
                && let Some(previous) = &tokens[index - 1]
                && (previous.text.contains("//")
                    || is_line_comment_prefix(&previous.text))
                && let Some(info) =
                    match_token(&format!("{}{}", previous.text, token.text), last)
            {
                comments.push(Comment {
                    info,
                    line,
                    token: index - 1,
                    line_comment_only: length + numbered == 2,
                    jsx_style: false,
                    additional: vec![index],
                });
                continue;
            }
            let Some(info) = found else { continue };
            if jsx && !last && index != 0 {
                let jsx_style = is_value(&tokens[index - 1], "{")
                    && is_value(&tokens[index + 1], "}");
                comments.push(Comment {
                    info,
                    line,
                    token: index,
                    line_comment_only: length + numbered == 3,
                    jsx_style,
                    additional: Vec::new(),
                });
            } else {
                comments.push(Comment {
                    info,
                    line,
                    token: index,
                    line_comment_only: length + numbered == 1,
                    jsx_style: false,
                    additional: Vec::new(),
                });
            }
        }
    }
    comments
}

/// A comment's marker, body and closing marker, the spaces around kept on the markers.
pub(crate) fn match_token(text: &str, last: bool) -> Option<[String; 3]> {
    // Shiki counts the space around in UTF-16 units and gives it back as plain spaces
    let trimmed_start = text.trim_start();
    let front = text[..text.len() - trimmed_start.len()]
        .encode_utf16()
        .count();
    let trimmed = trimmed_start.trim_end();
    let end = trimmed_start[trimmed.len()..].encode_utf16().count();
    for (matcher, end_of_line) in MATCHERS.iter() {
        if *end_of_line && !last {
            continue;
        }
        let Some(captures) = matcher.captures(trimmed) else {
            continue;
        };
        let group = |index| captures.get(index).map_or("", |found| found.as_str());
        let closing = match captures.get(3) {
            Some(found) => format!("{}{}", found.as_str(), " ".repeat(end)),
            None => String::new(),
        };
        let opening = format!("{}{}", " ".repeat(front), group(1));
        return Some([opening, group(2).to_owned(), closing]);
    }
    None
}

// v3 cuts a token holding a comment after code, `a // b`, at each ` //`
fn split_comments(tokens: &mut Vec<Option<CodeToken>>) {
    let Some(splitter) = SPLIT_COMMENT.as_ref() else {
        return;
    };
    let length = tokens.len();
    let mut split = Vec::with_capacity(length);
    for (index, slot) in std::mem::take(tokens).into_iter().enumerate() {
        let Some(token) = slot else {
            split.push(None);
            continue;
        };
        if match_token(&token.text, index == length - 1).is_none() {
            split.push(Some(token));
            continue;
        }
        let mut pieces = vec![];
        let mut cursor = 0;
        for found in splitter.find_iter(&token.text) {
            let before = &token.text[cursor..found.start()];
            match pieces.is_empty() {
                true => pieces.push(before.to_owned()),
                false => {
                    if let Some(last) = pieces.last_mut() {
                        last.push_str(before);
                    }
                }
            }
            pieces.push(found.as_str().to_owned());
            cursor = found.end();
        }
        if pieces.is_empty() {
            split.push(Some(token));
            continue;
        }
        if let Some(last) = pieces.last_mut() {
            last.push_str(&token.text[cursor..]);
        }
        pieces.retain(|piece| !piece.is_empty());
        if pieces.len() <= 1 {
            split.push(Some(token));
            continue;
        }
        for piece in pieces {
            split.push(Some(CodeToken {
                text: piece,
                style: token.style.clone(),
                classes: token.classes.clone(),
            }));
        }
    }
    *tokens = split;
}

fn is_line_comment_prefix(text: &str) -> bool {
    LINE_COMMENT_PREFIX
        .as_ref()
        .is_some_and(|prefix| prefix.is_match(text.trim()))
}

fn is_value(token: &Option<CodeToken>, value: &str) -> bool {
    token
        .as_ref()
        .is_some_and(|token| token.text.trim() == value)
}
