use std::sync::LazyLock;

use regex::Regex;

use crate::models::highlight::{CodeBlock, add_classes};
use crate::utils::notation::parse::Comment;

/// One notation transformer of @shikijs/transformers, as `transformerNotationMap` builds
/// it: each `[!code <key>]` adds the key's classes to its line, or to the lines `:n` spans.
pub(crate) struct Notation {
    pub(crate) class_map: Vec<(String, Vec<String>)>,
    pub(crate) active_pre: Vec<String>,
    pub(crate) active_code: Vec<String>,
}

static ANY_NOTATION: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?i)#?\s*\[!code\b(?:\\.|[^\]])*\]").ok());
static END_COMMENT_PREFIX: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?://|#|;{1,2}|%{1,2}|--)\s*$").ok());

/// Apply a notation transformer's `code` hook to the comments every notation transformer
/// of the block shares, as `createCommentNotationTransformer` does with matching `v3`.
pub(crate) fn apply_notation(
    block: &mut CodeBlock,
    comments: &mut [Comment],
    notation: &Notation,
) {
    let keys: Vec<String> = notation
        .class_map
        .iter()
        .map(|(key, _)| regex::escape(key))
        .collect();
    let Ok(pattern) =
        Regex::new(&format!(r"(?i)#?\s*\[!code ({})(:\d+)?\]", keys.join("|")))
    else {
        return;
    };
    let lines = block.visible_lines();
    let mut positions = vec![-1isize; block.lines.len()];
    for (position, &line) in lines.iter().enumerate() {
        positions[line] = position as isize;
    }
    let mut removed_lines = Vec::new();
    for comment in comments.iter_mut() {
        if comment.info[1].is_empty() {
            continue;
        }
        let mut index = positions[comment.line];
        if comment.line_comment_only && is_standalone(&comment.info[1], &pattern) {
            index += 1;
        }
        let matches: Vec<(String, usize)> = pattern
            .captures_iter(&comment.info[1])
            .map(|captures| {
                let key = captures.get(1).map_or("", |key| key.as_str()).to_owned();
                let range = captures.get(2).map_or(":1", |range| range.as_str());
                // `\d+` only fails to parse past the largest count, which spans every line
                (key, range[1..].parse().unwrap_or(usize::MAX))
            })
            .collect();
        if matches.is_empty() {
            continue;
        }
        for (key, count) in matches {
            let classes = notation.class_map.iter().find(|(name, _)| *name == key);
            let count = isize::try_from(count).unwrap_or(isize::MAX);
            let end = index.saturating_add(count).min(lines.len() as isize);
            for target in index.max(0)..end {
                if let Some((_, classes)) = classes {
                    add_classes(
                        &mut block.lines[lines[target as usize]].classes,
                        classes,
                    );
                }
            }
            add_classes(&mut block.pre_classes, &notation.active_pre);
            add_classes(&mut block.code_classes, &notation.active_code);
        }
        let mut body = pattern.replace_all(&comment.info[1], "").into_owned();
        if let Some(prefix) = END_COMMENT_PREFIX.as_ref().and_then(|end| end.find(&body))
        {
            body = body[..prefix.start()].trim_end().to_owned();
        }
        let empty = body.trim().is_empty();
        comment.info[1] = if empty { String::new() } else { body };
        let tokens = &mut block.lines[comment.line].tokens;
        if empty && comment.line_comment_only {
            removed_lines.push(comment.line);
        } else if empty && comment.jsx_style {
            let present: Vec<usize> =
                (0..tokens.len()).filter(|&i| tokens[i].is_some()).collect();
            if let Some(at) = present.iter().position(|&i| i == comment.token) {
                for &i in present.iter().skip(at.saturating_sub(1)).take(3) {
                    tokens[i] = None;
                }
            }
        } else if empty {
            for &extra in comment.additional.iter().rev() {
                tokens[extra] = None;
            }
            tokens[comment.token] = None;
        } else {
            if let Some(token) = &mut tokens[comment.token] {
                token.text = comment.info.concat();
            }
            for &extra in &comment.additional {
                if let Some(token) = &mut tokens[extra] {
                    token.text = String::new();
                }
            }
        }
    }
    for line in removed_lines {
        block.lines[line].removed = true;
    }
}

// A comment that is nothing but notations applies to the line after it
fn is_standalone(body: &str, pattern: &Regex) -> bool {
    let Some(any) = ANY_NOTATION.as_ref() else {
        return false;
    };
    let rest = any.replace_all(body, "");
    pattern.replace_all(&rest, "").trim().is_empty()
}
