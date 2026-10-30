use textlane_core::{Attach, Error, Plugin};

use std::sync::atomic::Ordering;

use crate::models::options::{Mdx, MdxSlot, UnravelSlot};

impl Plugin for Mdx {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        if let Some(quote) = self.quote.as_deref()
            && !quote.is_empty()
            && quote != "\""
            && quote != "'"
        {
            return Err(Error::new(
                "invalid-options",
                format!(
                    "Cannot serialize attribute values with `{quote}` for `options.quote`, expected `\"`, or `'`"
                ),
            ));
        }
        context.slots_mut().get_or_default::<MdxSlot>().0 = Some(self.clone());
        (context.slots_mut().get_or_default::<UnravelSlot>().0)
            .store(true, Ordering::Relaxed);
        Ok(())
    }
}
