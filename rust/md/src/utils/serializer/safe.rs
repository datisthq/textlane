use std::collections::HashMap;

use textlane_core::mdast::Node;

use crate::helpers::character::CharacterKind;
use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::state::{
    Serializer, classify_unit, encode_character_reference, read_node_type,
};
use crate::utils::serializer::unsafe_patterns::Pattern;

/// What surrounds a value `safe` escapes.
#[derive(Clone, Copy, Default)]
pub(crate) struct SafeConfig<'s, 'n, 'a> {
    pub(crate) before: &'s str,
    pub(crate) after: &'s str,
    pub(crate) before_node: Option<&'n Node<'a>>,
    pub(crate) after_node: Option<&'n Node<'a>>,
    /// Characters to write as references rather than with a backslash.
    pub(crate) encode: &'s [char],
}

#[derive(Clone, Copy)]
struct Found {
    before: bool,
    after: bool,
}

impl Serializer<'_> {
    /// Escape what in `input` would otherwise form syntax, as mdast-util-to-markdown's
    /// `safe` does.
    pub(crate) fn safe(&self, input: &str, config: &SafeConfig) -> String {
        let value = format!("{}{input}{}", config.before, config.after);
        let offset = config.before.len();
        let end = value.len() - config.after.len();
        let percent_encode = self.in_construct(Construct::Autolink);
        let mut positions = Vec::new();
        let mut infos: HashMap<usize, Found> = HashMap::new();
        for pattern in self.patterns.iter() {
            if !self.pattern_in_scope(pattern)
                || !is_node_in_scope(pattern.before_node.as_deref(), config.before_node)
                || !is_node_in_scope(pattern.after_node.as_deref(), config.after_node)
            {
                continue;
            }
            let from = if pattern.before_node.is_some() {
                offset
            } else {
                0
            };
            let to = if pattern.after_node.is_some() {
                end
            } else {
                value.len()
            };
            let subject = &value[from..to];
            if !subject.contains(pattern.character.as_str()) {
                continue;
            }
            for captures in pattern.expression.captures_iter(subject.as_bytes()) {
                let Some(whole) = captures.get(0) else {
                    continue;
                };
                let lead = captures.get(1).map_or(0, |group| group.len());
                let position = from + whole.start() + lead;
                let found = Found {
                    before: pattern.has_before,
                    after: pattern.has_after,
                };
                match infos.get_mut(&position) {
                    Some(info) => {
                        info.before = info.before && found.before;
                        info.after = info.after && found.after;
                    }
                    None => {
                        positions.push(position);
                        infos.insert(position, found);
                    }
                }
            }
        }
        positions.sort_unstable();

        let bytes = value.as_bytes();
        let mut result = String::with_capacity(value.len());
        let mut start = offset;
        let mut index = 0;
        while index < positions.len() {
            let position = positions[index];
            if position < start || position >= end {
                index += 1;
                continue;
            }
            // An intraword run of `_` cannot form emphasis, so it stays as it is
            if bytes[position] == b'_'
                && position > 0
                && classify_unit(value[..position].chars().next_back())
                    == CharacterKind::Other
                && !infos.contains_key(&(position - 1))
                && !(position - 1 == offset
                    && offset > 0
                    && matches!(bytes[offset - 1], b'*' | b'_'))
            {
                let mut sequence_end = position + 1;
                while sequence_end < end && bytes[sequence_end] == b'_' {
                    sequence_end += 1;
                }
                let skip = sequence_end - position - 1;
                if positions.get(index + skip) == Some(&(sequence_end - 1))
                    && sequence_end < value.len()
                    && classify_unit(value[sequence_end..].chars().next())
                        == CharacterKind::Other
                    && !infos.contains_key(&sequence_end)
                    && !(sequence_end + 1 == end
                        && matches!(bytes.get(end), Some(b'*' | b'_')))
                {
                    index += skip + 1;
                    continue;
                }
            }
            let info = infos[&position];
            let next_found = (position + 1 < end
                && positions.get(index + 1) == Some(&(position + 1)))
            .then(|| infos[&(position + 1)]);
            let previous_found =
                (index > 0 && position > offset && positions[index - 1] == position - 1)
                    .then(|| infos[&(position - 1)]);
            if next_found.is_some_and(|next| info.after && !next.before && !next.after)
                || previous_found.is_some_and(|previous| {
                    info.before && !previous.before && !previous.after
                })
            {
                index += 1;
                continue;
            }
            if start != position {
                let slice = &value[start..position];
                if percent_encode {
                    result.push_str(slice);
                } else {
                    result.push_str(&escape_backslashes(slice, "\\"));
                }
            }
            let character = value[position..].chars().next().unwrap_or_default();
            start = position + character.len_utf8();
            if percent_encode {
                let mut units = [0; 2];
                result
                    .push_str(&format!("%{:02X}", character.encode_utf16(&mut units)[0]));
            } else if character.is_ascii_punctuation()
                && !config.encode.contains(&character)
            {
                result.push('\\');
                result.push(character);
            } else {
                result.push_str(&encode_character_reference(character));
            }
            index += 1;
        }
        let rest = &value[start.min(end)..end];
        if percent_encode {
            result.push_str(rest);
        } else {
            result.push_str(&escape_backslashes(rest, config.after));
        }
        result
    }

    pub(crate) fn pattern_in_scope(&self, pattern: &Pattern) -> bool {
        let is_in = |list: &Option<Vec<Construct>>, none: bool| match list {
            None => none,
            Some(list) => list.iter().any(|construct| self.in_construct(*construct)),
        };
        is_in(&pattern.in_construct, true) && !is_in(&pattern.not_in_construct, false)
    }
}

fn is_node_in_scope(types: Option<&[String]>, node: Option<&Node>) -> bool {
    match (types, node) {
        (None, _) => true,
        (Some(_), None) => false,
        (Some(types), Some(node)) => {
            types.iter().any(|kind| kind == read_node_type(node))
        }
    }
}

// Double each backslash that would otherwise escape the character after it
fn escape_backslashes(value: &str, after: &str) -> String {
    let whole = format!("{value}{after}");
    let bytes = whole.as_bytes();
    let mut result = String::with_capacity(value.len());
    let mut start = 0;
    for index in 0..bytes.len() {
        if bytes[index] == b'\\'
            && bytes.get(index + 1).is_some_and(u8::is_ascii_punctuation)
        {
            let stop = index.min(value.len());
            if start < stop {
                result.push_str(&value[start..stop]);
            }
            result.push('\\');
            start = index;
        }
    }
    if start < value.len() {
        result.push_str(&value[start..]);
    }
    result
}
