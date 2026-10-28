use textlane_core::Error;
use textlane_core::mdast::LinkReference;

use crate::actions::image_reference::stringify::{close_reference, write_association};
use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::format::read_association;
use crate::utils::serializer::state::{Info, Parent, Serializer};

/// Write a link reference, `[text][label]`.
pub(crate) fn stringify_link_reference(
    state: &mut Serializer,
    reference: &LinkReference,
    node: Parent,
) -> Result<(), Error> {
    state.enter(Construct::LinkReference);
    state.enter(Construct::Label);
    let text = state.container_phrasing(
        node,
        Info {
            before: "[",
            after: "]",
        },
    );
    state.exit();
    let text = match text {
        Ok(text) => text,
        Err(error) => {
            state.exit();
            return Err(error);
        }
    };
    let mut value = format!("[{text}][");
    let label = read_association(reference.identifier, reference.label);
    let association = write_association(state, &value, &label);
    state.exit();
    close_reference(&mut value, reference.reference_type, &text, &association);
    state.write(&value);
    Ok(())
}
