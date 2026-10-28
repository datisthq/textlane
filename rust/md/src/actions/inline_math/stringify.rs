use std::collections::HashSet;

use textlane_core::mdast::InlineMath;

use crate::actions::inline_code::stringify::replace_at_breaks;
use crate::utils::serializer::state::Serializer;

/// Write math (text), with a fence no run of dollars inside matches; two dollars at least
/// unless single dollars form math.
pub(crate) fn stringify_inline_math(state: &mut Serializer, math: &InlineMath) {
    let mut value = math.value.to_string();
    let mut runs = HashSet::new();
    let mut current = 0;
    for byte in value.bytes().chain([0]) {
        if byte == b'$' {
            current += 1;
        } else if current > 0 {
            runs.insert(current);
            current = 0;
        }
    }
    let single = state
        .settings
        .math
        .as_ref()
        .is_none_or(|options| options.single_dollar_text_math);
    let mut size = if single { 1 } else { 2 };
    while runs.contains(&size) {
        size += 1;
    }
    let is_padding = |byte: u8| matches!(byte, b'\n' | b'\r' | b' ');
    let bytes = value.as_bytes();
    if bytes.iter().any(|byte| !is_padding(*byte))
        && ((bytes.first().is_some_and(|byte| is_padding(*byte))
            && bytes.last().is_some_and(|byte| is_padding(*byte)))
            || bytes.first() == Some(&b'$')
            || bytes.last() == Some(&b'$'))
    {
        value = format!(" {value} ");
    }
    let value = replace_at_breaks(state, value);
    let fence = "$".repeat(size);
    state.write(&format!("{fence}{value}{fence}"));
}
