use std::sync::LazyLock;

use regex::{Captures, Regex};

use crate::helpers::readability_table::{IRREGULAR, SINGULAR_RULES, UNCOUNTABLE};

static RULES: LazyLock<Vec<(Regex, &str)>> = LazyLock::new(|| {
    SINGULAR_RULES
        .iter()
        .filter_map(|(source, replacement)| {
            Some((Regex::new(source).ok()?, *replacement))
        })
        .collect()
});

/// The singular of a lowercase word, as pluralize 8.0.0's `singular`.
pub(crate) fn singularize_word(word: &str) -> String {
    if IRREGULAR.iter().any(|(single, _)| *single == word) {
        return word.to_string();
    }
    // A plural listed twice maps to its last single, as pluralize's object keeps it
    if let Some((single, _)) = IRREGULAR.iter().rev().find(|(_, plural)| *plural == word)
    {
        return (*single).to_string();
    }
    if word.is_empty() || UNCOUNTABLE.binary_search(&word).is_ok() {
        return word.to_string();
    }
    for (rule, replacement) in RULES.iter().rev() {
        if let Some(captures) = rule.captures(word) {
            let Some(found) = captures.get(0) else {
                break;
            };
            let mut result = String::with_capacity(word.len());
            result.push_str(&word[..found.start()]);
            result.push_str(&interpolate(replacement, &captures).to_lowercase());
            result.push_str(&word[found.end()..]);
            return result;
        }
    }
    word.to_string()
}

// `$n` is the nth group, or nothing when it did not take part
fn interpolate(replacement: &str, captures: &Captures<'_>) -> String {
    let mut result = String::new();
    let mut characters = replacement.chars().peekable();
    while let Some(character) = characters.next() {
        let digit = characters.peek().and_then(|next| next.to_digit(10));
        match (character, digit) {
            ('$', Some(digit)) => {
                characters.next();
                let mut group = digit as usize;
                if let Some(second) = characters.peek().and_then(|next| next.to_digit(10))
                {
                    characters.next();
                    group = group * 10 + second as usize;
                }
                if let Some(text) = captures.get(group) {
                    result.push_str(text.as_str());
                }
            }
            _ => result.push(character),
        }
    }
    result
}
