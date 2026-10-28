use textlane_core::Error;
use textlane_core::mdast::FootnoteDefinition;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::format::read_association;
use crate::utils::serializer::safe::SafeConfig;
use crate::utils::serializer::sink::Layer;
use crate::utils::serializer::state::{Parent, Serializer};

/// Write a GFM footnote definition, `[^a]:`, its content indented by four spaces after
/// its first line.
pub(crate) fn stringify_footnote_definition(
    state: &mut Serializer,
    definition: &FootnoteDefinition,
    node: Parent,
) -> Result<(), Error> {
    let mut value = "[^".to_string();
    state.enter(Construct::Label);
    let label = read_association(definition.identifier, definition.label);
    let safe = state.safe(
        &label,
        &SafeConfig {
            before: &value,
            after: "]",
            ..SafeConfig::default()
        },
    );
    state.exit();
    value.push_str(&safe);
    value.push_str("]:");
    if definition.children.is_empty() {
        state.write(&value);
        return Ok(());
    }
    let first_line_blank = state
        .settings
        .gfm
        .as_ref()
        .is_some_and(|gfm| gfm.first_line_blank);
    value.push(if first_line_blank { '\n' } else { ' ' });
    state.write(&value);
    state.open_layer(Layer::FootnoteDefinition {
        indent_first: first_line_blank,
    });
    let result = state.container_flow(node);
    state.close_layer();
    result
}
