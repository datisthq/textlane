use textlane_core::{Attach, Error, Plugin};

use crate::models::options::{GfmOptions, GfmSlot};

/// `md.gfm`: GitHub Flavored Markdown's syntax, read by `md::parse` and written by
/// `md::stringify`.
#[derive(Clone, Debug, Default)]
pub struct Gfm {
    pub options: GfmOptions,
}

/// The `md.gfm` plugin.
pub fn gfm(options: GfmOptions) -> Gfm {
    Gfm { options }
}

impl Plugin for Gfm {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.slots_mut().get_or_default::<GfmSlot>().0 = Some(self.options.clone());
        Ok(())
    }
}
