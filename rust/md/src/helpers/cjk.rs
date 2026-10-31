use crate::helpers::cjk_table::CJK_CLASSES;

const WHITESPACE: u16 = 1;
const PUNCTUATION: u16 = 2;
const SPACE_OR_PUNCTUATION: u16 = 3;
const CJK: u16 = 4096;
const CJK_PUNCTUATION: u16 = 4098;
const IVS: u16 = 8192;
const CJK_OR_IVS: u16 = 12288;
const VARIATION_SELECTOR: u16 = 16384;

/// Whether a run of `*`, `_` or `~` can open and close, as markdown-cjk-friendly decides it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Sides {
    pub(crate) open: bool,
    pub(crate) close: bool,
}

/// micromark-extension-cjk-friendly's attention tokenizer: `previous` and `two_previous` the
/// characters before the run, `next` the one after; `tilde` whether GFM's `~` is a marker.
pub(crate) fn find_attention_sides(
    marker: u8,
    previous: Option<char>,
    two_previous: impl FnOnce() -> Option<char>,
    next: Option<char>,
    tilde: bool,
) -> Sides {
    let before = classify_preceding(previous, two_previous);
    let after = classify(next);
    let marker_before = matches!(previous, Some('*' | '_'));
    let marker_after = matches!(next, Some('*' | '_'));
    let before_non_cjk = is_non_cjk_punctuation(before);
    let after_non_cjk = is_non_cjk_punctuation(after);
    let before_space = before_non_cjk || is_whitespace(before) || marker_before;
    let after_space = after_non_cjk || is_whitespace(after) || marker_after;
    let open = !after_space
        || (after_non_cjk && (before_space || before & CJK_OR_IVS != 0))
        || (tilde && next == Some('~'));
    let close = !before_space
        || (before_non_cjk && (after_space || after & CJK != 0))
        || (tilde && previous == Some('~'));
    match marker {
        b'*' => Sides { open, close },
        _ => Sides {
            open: open && (before & SPACE_OR_PUNCTUATION != 0 || !close),
            close: close && (after & SPACE_OR_PUNCTUATION != 0 || !open),
        },
    }
}

/// micromark-extension-cjk-friendly-gfm-strikethrough's tokenizer for a run of `~`.
pub(crate) fn find_strikethrough_sides(
    previous: Option<char>,
    two_previous: impl FnOnce() -> Option<char>,
    next: Option<char>,
) -> Sides {
    let plain_before = classify(previous);
    let before = classify_preceding(previous, two_previous);
    let after = classify(next);
    let before_space = is_non_cjk_punctuation(before) || is_whitespace(before);
    let after_space = is_non_cjk_punctuation(after) || is_whitespace(after);
    let before_cjk_or_ivs = before & CJK != 0 || plain_before == IVS;
    Sides {
        open: !after_space
            || (after == PUNCTUATION && (before_space || before_cjk_or_ivs)),
        close: !before_space
            || (plain_before == PUNCTUATION && (after_space || after & CJK != 0)),
    }
}

// `classifyCharacter`: the end of input is white space
fn classify(character: Option<char>) -> u16 {
    let Some(character) = character else {
        return WHITESPACE;
    };
    let code = u32::from(character);
    CJK_CLASSES
        .binary_search_by(|(first, last, _)| {
            if *last < code {
                std::cmp::Ordering::Less
            } else if *first > code {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .map_or(0, |index| CJK_CLASSES[index].2)
}

// `classifyPrecedingCharacter`: a variation selector takes the class of what it varies, or
// CJK punctuation for a curly quote with U+FE01, unless white space or nothing precedes it
fn classify_preceding(
    previous: Option<char>,
    two_previous: impl FnOnce() -> Option<char>,
) -> u16 {
    let before = classify(previous);
    if before != VARIATION_SELECTOR {
        return before;
    }
    let Some(two) = two_previous() else {
        return before;
    };
    let two_before = classify(Some(two));
    if is_whitespace(two_before) {
        before
    } else if previous == Some('\u{FE01}')
        && matches!(two, '\u{2018}' | '\u{2019}' | '\u{201C}' | '\u{201D}')
    {
        CJK_PUNCTUATION
    } else {
        two_before & !IVS
    }
}

fn is_whitespace(class: u16) -> bool {
    class & WHITESPACE != 0
}

fn is_non_cjk_punctuation(class: u16) -> bool {
    class & CJK_PUNCTUATION == PUNCTUATION
}
