use textlane_core::{Attach, Error, Plugin};

use crate::models::options::{Directive, DirectiveSlot};

impl Plugin for Directive {
    fn attach(&self, context: &mut Attach) -> Result<(), Error> {
        if let Some(quote) = self.quote.as_deref()
            && quote != "\""
            && quote != "'"
        {
            return Err(Error::new(
                "invalid-options",
                format!("Invalid quote `{quote}`, expected `'` or `\"`"),
            ));
        }
        context.slots_mut().get_or_default::<DirectiveSlot>().0 = Some(self.clone());
        Ok(())
    }
}
