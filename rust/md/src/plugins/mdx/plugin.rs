use textlane_core::{Attach, Error, Plugin};

use std::sync::atomic::Ordering;

use crate::models::options::{MdxOptions, MdxSlot, UnravelSlot};

/// `md.mdx`: MDX syntax, JSX, expressions and ESM, read by `md::parse` and written by
/// `md::stringify`; it turns off indented code, autolinks and raw HTML.
#[derive(Clone, Debug, Default)]
pub struct Mdx {
    pub options: MdxOptions,
}

/// The `md.mdx` plugin.
pub fn mdx(options: MdxOptions) -> Mdx {
    Mdx { options }
}

impl Plugin for Mdx {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        if let Some(quote) = self.options.quote.as_deref()
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
        context.slots_mut().get_or_default::<MdxSlot>().0 = Some(self.options.clone());
        (context.slots_mut().get_or_default::<UnravelSlot>().0)
            .store(true, Ordering::Relaxed);
        Ok(())
    }
}
