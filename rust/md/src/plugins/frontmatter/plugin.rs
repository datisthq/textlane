use textlane_core::{Attach, Error, Plugin};

use crate::models::options::{Frontmatter, FrontmatterSlot};

impl Plugin for Frontmatter {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        let slot = context.slots_mut().get_or_default::<FrontmatterSlot>();
        slot.0.extend(self.0.iter().cloned());
        Ok(())
    }
}
