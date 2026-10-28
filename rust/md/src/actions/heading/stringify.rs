use textlane_core::Error;
use textlane_core::mdast::Heading;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::format::format_heading_as_setext;
use crate::utils::serializer::state::{
    Info, Parent, Serializer, encode_character_reference,
};

/// Write a heading, ATX or setext.
pub(crate) fn stringify_heading(
    state: &mut Serializer,
    heading: &Heading,
    node: Parent,
) -> Result<(), Error> {
    let rank = usize::from(heading.depth.clamp(1, 6));
    if format_heading_as_setext(heading, state.settings) {
        state.enter(Construct::HeadingSetext);
        state.enter(Construct::Phrasing);
        let value = state.container_phrasing(
            node,
            Info {
                before: "\n",
                after: "\n",
            },
        );
        state.exit();
        state.exit();
        let value = value?;
        let last_line = &value[value.rfind(['\n', '\r']).map_or(0, |index| index + 1)..];
        let marker = if rank == 1 { "=" } else { "-" };
        state.write(&value);
        state.write("\n");
        state.write(&marker.repeat(last_line.encode_utf16().count()));
        return Ok(());
    }
    let sequence = "#".repeat(rank);
    state.enter(Construct::HeadingAtx);
    state.enter(Construct::Phrasing);
    let value = state.container_phrasing(
        node,
        Info {
            before: "# ",
            after: "\n",
        },
    );
    state.exit();
    state.exit();
    let mut value = value?;
    // A space or tab right after the sequence would be taken as the separator
    if let Some(head @ ('\t' | ' ')) = value.chars().next() {
        value = format!("{}{}", encode_character_reference(head), &value[1..]);
    }
    let mut result = sequence.clone();
    if !value.is_empty() {
        result.push(' ');
        result.push_str(&value);
    }
    if state.settings.close_atx {
        result.push(' ');
        result.push_str(&sequence);
    }
    state.write(&result);
    Ok(())
}
