use textlane_core::Error;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::state::{Info, Parent, Serializer};

/// Write a paragraph.
pub(crate) fn stringify_paragraph(
    state: &mut Serializer,
    paragraph: Parent,
    info: Info,
) -> Result<(), Error> {
    state.enter(Construct::Paragraph);
    state.enter(Construct::Phrasing);
    let value = state.container_phrasing(paragraph, info);
    state.exit();
    state.exit();
    state.write(&value?);
    Ok(())
}
