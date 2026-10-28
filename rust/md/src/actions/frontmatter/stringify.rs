use textlane_core::Error;

use crate::models::options::{Matter, UnsafePattern};
use crate::utils::serializer::state::Serializer;

/// Write frontmatter of `kind` between its matter's fences; the last matter of a kind
/// wins, as a later handler replaces an earlier one upstream.
pub(crate) fn stringify_frontmatter(
    state: &mut Serializer,
    kind: &str,
    value: &str,
) -> Result<(), Error> {
    let Some(matter) = state
        .settings
        .frontmatter
        .iter()
        .rev()
        .find(|matter| matter.kind == kind)
    else {
        return Err(Error::new(
            "unknown-node",
            format!("Cannot handle unknown node `{kind}`"),
        ));
    };
    let mut output = matter.open.clone();
    if !value.is_empty() {
        output.push('\n');
        output.push_str(value);
    }
    output.push('\n');
    output.push_str(&matter.close);
    state.write(&output);
    Ok(())
}

/// The unsafe patterns that keep text from reading as each matter's opening fence.
pub(crate) fn create_frontmatter_patterns(matters: &[Matter]) -> Vec<UnsafePattern> {
    matters
        .iter()
        .filter_map(|matter| {
            let mut characters = matter.open.chars();
            let first = characters.next()?;
            let after = characters
                .next()
                .map(|next| regex::escape(next.encode_utf8(&mut [0; 4])));
            Some(UnsafePattern {
                character: first.to_string(),
                at_break: true,
                after: Some(after.unwrap_or_default()),
                ..UnsafePattern::default()
            })
        })
        .collect()
}
