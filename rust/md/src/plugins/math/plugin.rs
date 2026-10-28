use textlane_core::{Attach, Error, Plugin};

use crate::models::options::{MathOptions, MathSlot};

/// `md.math`: math syntax, `$$` blocks and `$` text, read by `md::parse` and written by
/// `md::stringify`.
#[derive(Clone, Debug, Default)]
pub struct Math {
    pub options: MathOptions,
}

/// The `md.math` plugin.
pub fn math(options: MathOptions) -> Math {
    Math { options }
}

impl Plugin for Math {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.slots_mut().get_or_default::<MathSlot>().0 = Some(self.options.clone());
        Ok(())
    }
}
