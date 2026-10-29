use crate::helpers::unicode::{
    AFFIX_SYMBOL, LOWER_INITIAL, NUMERICAL, PUNCTUATION, WHITE_SPACE, WORD,
};

/// What run of the tokenizer a character starts or continues, in parse-latin's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Class {
    WhiteSpace,
    Punctuation,
    Word,
    Symbol,
}

/// The class parse-latin gives a character, testing each of its UTF-16 units.
pub(crate) fn classify_character(character: char) -> Class {
    if is_in(WHITE_SPACE, character) {
        Class::WhiteSpace
    } else if is_in(PUNCTUATION, character) {
        Class::Punctuation
    } else if is_in(WORD, character) {
        Class::Word
    } else {
        Class::Symbol
    }
}

/// Whether `value` is one character `numerical` matches.
pub(crate) fn is_numerical(value: &str) -> bool {
    let mut characters = value.chars();
    matches!((characters.next(), characters.next()), (Some(first), None) if is_in(NUMERICAL, first))
}

/// Whether `value` is a run of one closing or final mark `affixSymbol` matches.
pub(crate) fn is_affix_symbol(value: &str) -> bool {
    let mut characters = value.chars();
    match characters.next() {
        Some(first) => {
            is_in(AFFIX_SYMBOL, first) && characters.all(|other| other == first)
        }
        None => false,
    }
}

/// Whether a word starting with `character` starts lower case, as `lowerInitial` tests it.
pub(crate) fn is_lower_initial(character: char) -> bool {
    is_in(LOWER_INITIAL, character)
}

/// Whether `value` is one or more terminal markers: `.`, `?`, `!`, `…` or `‽`.
pub(crate) fn is_terminal_marker(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| matches!(character, '!' | '.' | '?' | '…' | '‽'))
}

/// Whether `value` is a mark that joins two words: `wordSymbolInner`.
pub(crate) fn is_word_symbol_inner(value: &str) -> bool {
    let mut characters = value.chars();
    match (characters.next(), characters.next()) {
        (
            Some(
                '&' | '\'' | '-' | '.' | ':' | '=' | '?' | '@' | '\u{AD}' | '\u{B7}'
                | '\u{2010}' | '\u{2011}' | '\u{2019}' | '\u{2027}',
            ),
            None,
        ) => true,
        (Some('_'), _) => value.bytes().all(|byte| byte == b'_'),
        _ => false,
    }
}

/// Whether white space is a line break: line endings with only spaces and tabs around them.
pub(crate) fn is_new_line(value: &str) -> bool {
    value.contains(['\n', '\r'])
        && value
            .chars()
            .all(|character| matches!(character, ' ' | '\t' | '\n' | '\r'))
}

/// How many line endings `value` holds, `\r\n` counting once.
pub(crate) fn count_line_endings(value: &str) -> usize {
    let bytes = value.as_bytes();
    let mut count = 0;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\r' => {
                count += 1;
                if bytes.get(index + 1) == Some(&b'\n') {
                    index += 1;
                }
            }
            b'\n' => count += 1,
            _ => {}
        }
        index += 1;
    }
    count
}

fn is_in(table: &[(char, char)], character: char) -> bool {
    table
        .binary_search_by(|(start, end)| {
            if *end < character {
                std::cmp::Ordering::Less
            } else if *start > character {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}
