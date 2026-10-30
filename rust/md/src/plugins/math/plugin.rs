use textlane_core::{Attach, Error, Plugin};

use crate::models::options::{Math, MathSlot};

impl Plugin for Math {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.slots_mut().get_or_default::<MathSlot>().0 = Some(self.clone());
        Ok(())
    }
}
