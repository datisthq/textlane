use std::collections::HashSet;

use textlane_core::mdast::InlineCode;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::state::Serializer;

/// Write a code span, with a fence no run of backticks inside matches.
pub(crate) fn stringify_inline_code(state: &mut Serializer, code: &InlineCode) {
    let mut value = code.value.to_string();
    let mut runs = HashSet::new();
    let mut current = 0;
    for byte in value.bytes().chain([0]) {
        if byte == b'`' {
            current += 1;
        } else if current > 0 {
            runs.insert(current);
            current = 0;
        }
    }
    let mut size = 1;
    while runs.contains(&size) {
        size += 1;
    }
    let is_padding = |byte: u8| matches!(byte, b'\n' | b'\r' | b' ');
    let bytes = value.as_bytes();
    if bytes.iter().any(|byte| !is_padding(*byte))
        && ((bytes.first().is_some_and(|byte| is_padding(*byte))
            && bytes.last().is_some_and(|byte| is_padding(*byte)))
            || bytes.first() == Some(&b'`')
            || bytes.last() == Some(&b'`'))
    {
        value = format!(" {value} ");
    }
    value = replace_at_breaks(state, value);
    // A pipe would end the cell, as GFM splits rows before code spans
    if state.in_construct(Construct::TableCell) {
        value = value.replace('|', "\\|");
    }
    let fence = "`".repeat(size);
    state.write(&format!("{fence}{value}{fence}"));
}

/// Turn each line ending that would start a block construct inside a span into a space.
pub(crate) fn replace_at_breaks(state: &Serializer, mut value: String) -> String {
    for pattern in state.patterns.iter().filter(|pattern| pattern.at_break) {
        if !value.contains(pattern.character.as_str()) {
            continue;
        }
        let mut output = String::with_capacity(value.len());
        let mut copied = 0;
        let mut search = 0;
        while let Some(found) = pattern.expression.find_at(value.as_bytes(), search) {
            output.push_str(&value[copied..found.start()]);
            let mut removed = 0;
            if value.as_bytes()[found.start()] == b'\n' && output.ends_with('\r') {
                output.pop();
                removed = 1;
            }
            output.push(' ');
            copied = found.start() + 1;
            search = found.end() + removed;
            if search > value.len() {
                break;
            }
        }
        output.push_str(&value[copied.min(value.len())..]);
        value = output;
    }
    value
}
