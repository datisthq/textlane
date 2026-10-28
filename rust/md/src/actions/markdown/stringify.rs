use textlane_core::Error;
use textlane_core::mdast::Node;

use crate::actions::root::stringify::stringify_root;
use crate::models::options::StringifyOptions;
use crate::utils::serializer::format::is_phrasing;
use crate::utils::serializer::settings::Settings;
use crate::utils::serializer::state::{Info, Parent, Serializer};
use crate::utils::serializer::unsafe_patterns::{Pattern, compile_patterns};

/// Serialize mdast as Markdown, as mdast-util-to-markdown does.
pub fn stringify_markdown(
    tree: &Node,
    options: &StringifyOptions,
) -> Result<String, Error> {
    let settings = Settings::from_options(options)?;
    let patterns = compile_patterns(false, &options.unsafe_patterns)?;
    serialize_tree(tree, &settings, &patterns)
}

pub(crate) fn serialize_tree(
    tree: &Node,
    settings: &Settings,
    patterns: &[Pattern],
) -> Result<String, Error> {
    let mut state = Serializer::new(settings, patterns);
    let info = Info {
        before: "\n",
        after: "\n",
    };
    if is_phrasing(tree) {
        let root = Parent {
            node: None,
            children: std::slice::from_ref(tree),
        };
        stringify_root(&mut state, root, info)?;
    } else {
        state.handle(tree, None, info)?;
    }
    let mut result = state.finish();
    if !result.is_empty() && !result.ends_with(['\n', '\r']) {
        result.push('\n');
    }
    Ok(result)
}
