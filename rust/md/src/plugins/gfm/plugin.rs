use textlane_core::{Attach, Error, Plugin};

use crate::models::options::{Gfm, GfmSlot};

impl Plugin for Gfm {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.slots_mut().get_or_default::<GfmSlot>().0 = Some(self.clone());
        Ok(())
    }
}
