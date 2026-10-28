use textlane_core::{Attach, Error, Plugin};

use crate::models::options::{DirectiveOptions, DirectiveSlot};

/// `md.directive`: generic directives, `:a`, `::b` and `:::c`, read by `md::parse` and
/// written by `md::stringify`.
#[derive(Clone, Debug, Default)]
pub struct Directive {
    pub options: DirectiveOptions,
}

/// The `md.directive` plugin.
pub fn directive(options: DirectiveOptions) -> Directive {
    Directive { options }
}

impl Plugin for Directive {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        if let Some(quote) = self.options.quote.as_deref()
            && quote != "\""
            && quote != "'"
        {
            return Err(Error::new(
                "invalid-options",
                format!("Invalid quote `{quote}`, expected `'` or `\"`"),
            ));
        }
        context.slots_mut().get_or_default::<DirectiveSlot>().0 =
            Some(self.options.clone());
        Ok(())
    }
}
