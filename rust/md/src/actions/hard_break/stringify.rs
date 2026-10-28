use crate::utils::serializer::state::{Info, Serializer};

/// Write a hard break: a backslash before the line ending, or a space where a line ending
/// cannot go.
pub(crate) fn stringify_hard_break(state: &mut Serializer, info: Info) {
    let unsafe_line_ending = state.patterns.iter().any(|pattern| {
        pattern.character == "\n" && pattern.is_bare && state.pattern_in_scope(pattern)
    });
    let value = if !unsafe_line_ending {
        "\\\n"
    } else if info.before.contains(['\t', ' ']) {
        ""
    } else {
        " "
    };
    state.write(value);
}
