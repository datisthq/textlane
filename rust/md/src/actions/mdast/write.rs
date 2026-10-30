use std::cell::OnceCell;

use textlane_core::mdast;

use crate::models::options::ToHtml;
use crate::utils::writer::state::Writer;

/// Write HTML from mdast as `md.toHtml` and then `html.stringify` would, without building
/// hast; `None` for a tree holding nodes left to those stages.
pub fn write_html(
    node: &mdast::Node<'_>,
    options: &ToHtml,
    allow_dangerous_html: bool,
) -> Option<String> {
    let mut writer = Writer {
        options,
        dangerous: allow_dangerous_html,
        root: node,
        definitions: OnceCell::new(),
        output: String::new(),
        depth: 0,
    };
    writer.write_root().ok()?;
    Some(writer.output)
}
