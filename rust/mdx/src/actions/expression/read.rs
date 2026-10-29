use textlane_core::Error;

use crate::actions::script::parse::parse_script;
use crate::models::estree::Id;
use crate::models::estree::{Store, Value};
use crate::models::name::Key;

/// Parse the JavaScript of an MDX expression, as micromark-util-events-to-acorn does: no
/// expression when it holds only white space and comments, else the one expression, read
/// as `({…})` with `spread`. Offsets are clamped to the source, which starts at `base`
/// (at least 2). Gives its comments too.
pub(crate) fn read_expression(
    store: &mut Store,
    source: &str,
    base: usize,
    spread: bool,
) -> Result<(Option<Id>, Vec<Id>), Error> {
    if is_empty(source) {
        let (_, comments) = parse_script(store, source, base)?;
        return Ok((None, comments));
    }
    // Parentheses make the source one expression statement; the line ending keeps a
    // trailing line comment from swallowing the closing one
    let (prefix, suffix) = if spread { ("({", "\n})") } else { ("(", "\n)") };
    let wrapped = format!("{prefix}{source}{suffix}");
    let first = store.len();
    let (program, comments) = parse_script(store, &wrapped, base - prefix.len())?;
    let (low, high) = (base as f64, (base + source.len()) as f64);
    for index in first..store.len() {
        let id = Id(index);
        for key in [Key::START, Key::END] {
            if let Some(offset) = store.number(id, key) {
                store.set(id, key, Value::Number(offset.clamp(low, high)));
            }
        }
    }
    let statement = store.children(program, Key::BODY).first().copied();
    Ok((
        statement.and_then(|statement| store.child(statement, Key::EXPRESSION)),
        comments,
    ))
}

// Upstream's `empty`: nothing but white space once block comments and line comments ended
// by a line ending are gone
fn is_empty(source: &str) -> bool {
    let mut rest = source;
    loop {
        rest = rest.trim_start();
        if let Some(after) = rest.strip_prefix("/*") {
            match after.find("*/") {
                Some(end) => rest = &after[end + 2..],
                None => return false,
            }
        } else if let Some(after) = rest.strip_prefix("//") {
            match after.find(['\n', '\r']) {
                Some(end) => rest = &after[end..],
                None => return false,
            }
        } else {
            return rest.is_empty();
        }
    }
}
