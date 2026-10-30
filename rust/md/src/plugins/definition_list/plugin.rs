use textlane_core::{Attach, Error, Plugin};

use crate::models::options::DefinitionListSlot;

/// `md.definitionList`: definition lists, read by `md::Parse` and written by
/// `md::Stringify`.
#[derive(Clone, Debug, Default)]
pub struct DefinitionList;

impl Plugin for DefinitionList {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.slots_mut().get_or_default::<DefinitionListSlot>().0 = true;
        Ok(())
    }
}
