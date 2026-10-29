use oxc_diagnostics::OxcDiagnostic;
use textlane_core::ScriptError;

// oxc recurses once per nesting level and has no depth guard of its own: every level needs
// a punctuation byte or a word, so their count bounds the depth, whatever the syntax
const LEVEL_BOUND_MAX: usize = 256 * 1024;
const LEVELS_SHALLOW: usize = 32;
// A debug build measured about 7.6 KiB per level of nested JSX; the rest is margin
const STACK_PER_LEVEL: usize = 16 * 1024;
const STACK_BASE: usize = 1024 * 1024;

/// The first of oxc's diagnostics as a script error, its offsets moved back by `shift`
/// and kept inside `0..=len`; `fatal` says the parser stopped at it, else it read all.
pub(crate) fn read_diagnostic(
    diagnostics: &[OxcDiagnostic],
    fatal: bool,
    shift: usize,
    len: usize,
) -> ScriptError {
    let first = diagnostics.iter().min_by_key(|diagnostic| {
        diagnostic
            .labels
            .first()
            .map_or(usize::MAX, |label| label.offset() as usize)
    });
    let reason = first.map_or_else(
        || "Unexpected token".into(),
        |error| error.message.to_string(),
    );
    let (start, end) = first
        .and_then(|diagnostic| diagnostic.labels.first())
        .map_or((len, len), |label| {
            let offset = label.offset() as usize;
            let start = offset.saturating_sub(shift).min(len);
            let end = (offset + label.len() as usize)
                .saturating_sub(shift)
                .min(len);
            (start, end)
        });
    ScriptError {
        start,
        end: if fatal { end } else { len },
        reason,
    }
}

/// An upper bound on how deep `source` can make oxc recurse; fails when too large to parse
/// safely.
pub(crate) fn check_nesting(source: &str) -> Result<usize, ScriptError> {
    let mut bound: usize = 0;
    let mut in_word = false;
    for (index, byte) in source.bytes().enumerate() {
        // Any non-ASCII character counts alone: some are separators (U+00A0, U+2028) that
        // would otherwise join `new\u{a0}new` into one word
        let word = byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$');
        let lead = byte >= 0xC0;
        if (word && !in_word) || byte.is_ascii_punctuation() || lead {
            bound += 1;
            if bound > LEVEL_BOUND_MAX {
                return Err(ScriptError {
                    start: index,
                    end: index + 1,
                    reason: "Too deeply nested".into(),
                });
            }
        }
        in_word = word;
    }
    Ok(bound)
}

/// Run `parse` with room on the stack for oxc's recursive descent `depth` levels deep.
pub(crate) fn with_stack<T>(depth: usize, parse: impl FnOnce() -> T) -> T {
    if depth <= LEVELS_SHALLOW {
        parse()
    } else {
        // A fresh stack per expression costs a map and its page faults: grow only when short
        let size = STACK_BASE + depth * STACK_PER_LEVEL;
        stacker::maybe_grow(size, size, parse)
    }
}
