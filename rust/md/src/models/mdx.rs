use std::fmt;
use std::sync::Arc;

use textlane_core::ScriptParser;

/// The JavaScript parser `md.mdx` validates expressions and ESM with, when there is one.
#[derive(Clone, Default)]
pub(crate) struct MdxScript(pub(crate) Option<Arc<dyn ScriptParser>>);

impl fmt::Debug for MdxScript {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(if self.0.is_some() {
            "MdxScript(Some)"
        } else {
            "MdxScript(None)"
        })
    }
}
