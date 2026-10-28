use crate::helpers::entities::CHARACTER_REFERENCES;
use crate::models::parse_error::ErrorCode;
use crate::utils::tokenizer::preprocess::{is_control, is_noncharacter};

/// Whether a reference sits in an attribute value, where a legacy reference followed by
/// `=` or an alphanumeric is left alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DecodeMode {
    Legacy,
    Attribute,
}

/// What `entities`' `EntityDecoder` makes of the text after a `&`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Reference {
    /// Bytes consumed from the `&` on, zero when the text is no reference.
    pub(crate) consumed: usize,
    pub(crate) chars: [Option<char>; 2],
    /// Errors, each at a byte offset from the `&`.
    pub(crate) errors: [Option<(ErrorCode, usize)>; 2],
}

// C1 controls a numeric reference maps to Windows-1252, as `entities`' `replaceCodePoint`
static C1_REPLACEMENTS: [(u32, char); 27] = [
    (0x80, '\u{20ac}'),
    (0x82, '\u{201a}'),
    (0x83, '\u{0192}'),
    (0x84, '\u{201e}'),
    (0x85, '\u{2026}'),
    (0x86, '\u{2020}'),
    (0x87, '\u{2021}'),
    (0x88, '\u{02c6}'),
    (0x89, '\u{2030}'),
    (0x8a, '\u{0160}'),
    (0x8b, '\u{2039}'),
    (0x8c, '\u{0152}'),
    (0x8e, '\u{017d}'),
    (0x91, '\u{2018}'),
    (0x92, '\u{2019}'),
    (0x93, '\u{201c}'),
    (0x94, '\u{201d}'),
    (0x95, '\u{2022}'),
    (0x96, '\u{2013}'),
    (0x97, '\u{2014}'),
    (0x98, '\u{02dc}'),
    (0x99, '\u{2122}'),
    (0x9a, '\u{0161}'),
    (0x9b, '\u{203a}'),
    (0x9c, '\u{0153}'),
    (0x9e, '\u{017e}'),
    (0x9f, '\u{0178}'),
];

const NONE: Reference = Reference {
    consumed: 0,
    chars: [None, None],
    errors: [None, None],
};

/// Decodes the reference that `rest`, the input after a `&`, starts with.
pub(crate) fn decode_reference(rest: &str, mode: DecodeMode) -> Reference {
    let bytes = rest.as_bytes();
    match bytes.first() {
        Some(b'#') => decode_numeric(&bytes[1..]),
        _ => decode_named(bytes, mode),
    }
}

fn decode_numeric(bytes: &[u8]) -> Reference {
    let hex = matches!(bytes.first(), Some(b'x' | b'X'));
    let (radix, prefix) = if hex { (16, 3) } else { (10, 2) };
    let digits_start = prefix - 2;
    let digits = bytes[digits_start.min(bytes.len())..]
        .iter()
        .take_while(|byte| char::from(**byte).is_digit(radix))
        .count();
    if digits == 0 {
        let error = (
            ErrorCode::AbsenceOfDigitsInNumericCharacterReference,
            prefix,
        );
        return Reference {
            errors: [Some(error), None],
            ..NONE
        };
    }
    // Saturates past the Unicode range, where every value decodes alike
    let code =
        bytes[digits_start..digits_start + digits]
            .iter()
            .fold(0u32, |code, byte| {
                let digit = char::from(*byte).to_digit(radix).unwrap_or(0);
                code.saturating_mul(radix)
                    .saturating_add(digit)
                    .min(0x11_0000)
            });
    let mut consumed = prefix + digits;
    let terminated = bytes.get(digits_start + digits) == Some(&b';');
    if terminated {
        consumed += 1;
    }
    let missing = (!terminated)
        .then_some((ErrorCode::MissingSemicolonAfterCharacterReference, consumed));
    let invalid = find_numeric_error(code).map(|error| (error, consumed));
    let errors = match missing {
        Some(missing) => [Some(missing), invalid],
        None => [invalid, None],
    };
    Reference {
        consumed,
        chars: [Some(replace_code_point(code)), None],
        errors,
    }
}

fn replace_code_point(code: u32) -> char {
    if code == 0 {
        return char::REPLACEMENT_CHARACTER;
    }
    if let Ok(index) = C1_REPLACEMENTS.binary_search_by_key(&code, |(from, _)| *from) {
        return C1_REPLACEMENTS[index].1;
    }
    char::from_u32(code).unwrap_or(char::REPLACEMENT_CHARACTER)
}

fn find_numeric_error(code: u32) -> Option<ErrorCode> {
    if code == 0 {
        Some(ErrorCode::NullCharacterReference)
    } else if code > 0x10_ffff {
        Some(ErrorCode::CharacterReferenceOutsideUnicodeRange)
    } else if (0xd800..=0xdfff).contains(&code) {
        Some(ErrorCode::SurrogateCharacterReference)
    } else if is_noncharacter(code) {
        Some(ErrorCode::NoncharacterCharacterReference)
    } else if is_control(code) || code == 0x0d {
        Some(ErrorCode::ControlCharacterReference)
    } else {
        None
    }
}

/// Walks the names that start with the text read so far, as `entities` walks its trie:
/// the longest name without `;` is kept while a longer one may still match.
fn decode_named(bytes: &[u8], mode: DecodeMode) -> Reference {
    let (mut low, mut high) = (0, CHARACTER_REFERENCES.len());
    let mut legacy: Option<(usize, &str)> = None;
    let mut at_legacy = false;
    let mut depth = 0;
    loop {
        let byte = bytes.get(depth).copied();
        let range = byte.map(|byte| narrow_range(low, high, depth, byte));
        let Some((next_low, next_high)) = range.filter(|(low, high)| low < high) else {
            let Some((length, value)) = legacy else {
                return NONE;
            };
            let invalid_end =
                byte.is_some_and(|byte| byte == b'=' || byte.is_ascii_alphanumeric());
            if mode == DecodeMode::Attribute && (!at_legacy || invalid_end) {
                return NONE;
            }
            let consumed = length + 1;
            let error = (ErrorCode::MissingSemicolonAfterCharacterReference, consumed);
            return Reference {
                consumed,
                chars: split_value(value),
                errors: [Some(error), None],
            };
        };
        (low, high, depth) = (next_low, next_high, depth + 1);
        let (name, value) = CHARACTER_REFERENCES[low];
        at_legacy = false;
        if name.len() == depth {
            if byte == Some(b';') {
                return Reference {
                    consumed: depth + 1,
                    chars: split_value(value),
                    errors: [None, None],
                };
            }
            legacy = Some((depth, value));
            at_legacy = true;
        }
    }
}

/// The names in `low..high`, which share their first `depth` bytes, whose next is `byte`.
fn narrow_range(low: usize, high: usize, depth: usize, byte: u8) -> (usize, usize) {
    let names = &CHARACTER_REFERENCES[low..high];
    let at = |name: &str| name.as_bytes().get(depth).copied();
    let start =
        names.partition_point(|(name, _)| at(name).is_none_or(|each| each < byte));
    let end = names.partition_point(|(name, _)| at(name).is_none_or(|each| each <= byte));
    (low + start, low + end)
}

fn split_value(value: &str) -> [Option<char>; 2] {
    let mut chars = value.chars();
    [chars.next(), chars.next()]
}
