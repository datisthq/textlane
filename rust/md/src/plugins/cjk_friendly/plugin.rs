use textlane_core::{Attach, Error, Plugin};

use crate::models::options::CjkFriendlySlot;

/// `md.cjkFriendly`: `*`, `_` and, with `md.gfm`, `~` pair next to Chinese, Japanese and
/// Korean text and punctuation, as remark-cjk-friendly and its strikethrough companion parse.
#[derive(Clone, Copy, Debug, Default)]
pub struct CjkFriendly;

impl Plugin for CjkFriendly {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        context.slots_mut().get_or_default::<CjkFriendlySlot>().0 = true;
        Ok(())
    }
}
