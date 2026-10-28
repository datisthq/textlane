use textlane_core::{Attach, Error, Plugin};

use crate::models::options::{FrontmatterOptions, FrontmatterSlot};

/// `md.frontmatter`: YAML, TOML or custom matter at the start of a document, read by
/// `md::parse` and written by `md::stringify`.
#[derive(Clone, Debug, Default)]
pub struct Frontmatter {
    pub options: FrontmatterOptions,
}

/// The `md.frontmatter` plugin.
pub fn frontmatter(options: FrontmatterOptions) -> Frontmatter {
    Frontmatter { options }
}

impl Plugin for Frontmatter {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        let slot = context.slots_mut().get_or_default::<FrontmatterSlot>();
        slot.0.extend(self.options.0.iter().cloned());
        Ok(())
    }
}
