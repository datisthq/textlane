use textlane_core::{Attach, Error, Plugin};

use crate::models::options::DefinitionListSlot;

/// `md.definitionList`: definition lists, read by `md::parse` and written by
/// `md::stringify`.
#[derive(Clone, Debug, Default)]
pub struct DefinitionList;

/// The `md.definitionList` plugin.
pub fn definition_list() -> DefinitionList {
    DefinitionList
}

impl Plugin for DefinitionList {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.slots_mut().get_or_default::<DefinitionListSlot>().0 = true;
        Ok(())
    }
}
