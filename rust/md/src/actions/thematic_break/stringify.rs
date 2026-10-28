use crate::utils::serializer::state::Serializer;

/// Write a thematic break.
pub(crate) fn stringify_thematic_break(state: &mut Serializer) {
    let settings = state.settings;
    let mut unit = settings.rule.to_string();
    if settings.rule_spaces {
        unit.push(' ');
    }
    let mut value = unit.repeat(settings.rule_repetition);
    if settings.rule_spaces {
        value.pop();
    }
    state.write(&value);
}
