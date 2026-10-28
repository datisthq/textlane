use textlane_core::ArenaString;
use textlane_core::hast;
use textlane_core::mdast::Text;

use crate::utils::converter::state::Converter;

/// Text stays text, without the spaces and tabs around its line endings.
pub(crate) fn convert_text<'a>(
    converter: &mut Converter<'a, '_>,
    node: &Text<'a>,
) -> hast::Node<'a> {
    hast::Node::Text(hast::Text {
        value: trim_lines(converter, node.value),
        position: node.position,
        data: None,
    })
}

// As the trim-lines package does: lines keep their outer edges, the first its start and the
// last its end
fn trim_lines<'a>(converter: &Converter<'a, '_>, value: &'a str) -> &'a str {
    if !value.contains(['\n', '\r']) {
        return value;
    }
    let mut result = ArenaString::with_capacity_in(value.len(), converter.alloc);
    let bytes = value.as_bytes();
    let mut last = 0;
    let mut index = 0;
    while index < bytes.len() {
        let ending = match bytes[index] {
            b'\r' if bytes.get(index + 1) == Some(&b'\n') => 2,
            b'\r' | b'\n' => 1,
            _ => {
                index += 1;
                continue;
            }
        };
        result.push_str(trim_line(&value[last..index], last > 0, true));
        result.push_str(&value[index..index + ending]);
        index += ending;
        last = index;
    }
    result.push_str(trim_line(&value[last..], last > 0, false));
    result.into_bump_str()
}

fn trim_line(line: &str, start: bool, end: bool) -> &str {
    let is_blank = |c: char| c == ' ' || c == '\t';
    let line = if start {
        line.trim_start_matches(is_blank)
    } else {
        line
    };
    if end {
        line.trim_end_matches(is_blank)
    } else {
        line
    }
}
