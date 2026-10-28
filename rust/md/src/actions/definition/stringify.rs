use textlane_core::mdast::Definition;

use crate::actions::image::stringify::{
    has_control_or_space, write_literal_destination, write_title,
};
use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::format::read_association;
use crate::utils::serializer::safe::SafeConfig;
use crate::utils::serializer::state::Serializer;

/// Write a definition, `[label]: url "title"`.
pub(crate) fn stringify_definition(state: &mut Serializer, definition: &Definition) {
    let title = definition.title.filter(|title| !title.is_empty());
    state.enter(Construct::Definition);
    state.enter(Construct::Label);
    let mut value = "[".to_string();
    let label = read_association(definition.identifier, definition.label);
    let safe = state.safe(
        &label,
        &SafeConfig {
            before: &value,
            after: "]",
            ..SafeConfig::default()
        },
    );
    value.push_str(&safe);
    value.push_str("]: ");
    state.exit();
    let url = definition.url;
    if url.is_empty() || has_control_or_space(url) {
        write_literal_destination(state, &mut value, url);
    } else {
        state.enter(Construct::DestinationRaw);
        let after = if title.is_some() { " " } else { "\n" };
        let safe = state.safe(
            url,
            &SafeConfig {
                before: &value,
                after,
                ..SafeConfig::default()
            },
        );
        value.push_str(&safe);
        state.exit();
    }
    if let Some(title) = title {
        write_title(state, &mut value, title);
    }
    state.exit();
    state.write(&value);
}
