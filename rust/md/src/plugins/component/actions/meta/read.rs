use std::sync::LazyLock;

use regex::Regex;

static PAIR: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r#"\b([A-Za-z_][\w-]*)=(?:"([^"]*)"|'([^']*)'|([^\s"']+))"#).ok()
});

/// The value of the first `key="value"` (or `'value'`, or a bare value) in a code meta.
pub(crate) fn read_meta(meta: &str, key: &str) -> Option<String> {
    find_pair(meta, key).map(|(value, _)| value)
}

/// A code meta without the first pair of each key, trimmed; `None` once nothing is left.
pub(crate) fn strip_meta(meta: &str, keys: &[&str]) -> Option<String> {
    let mut meta = meta.to_owned();
    for key in keys {
        if let Some((_, range)) = find_pair(&meta, key) {
            meta.replace_range(range, "");
        }
    }
    let meta = meta.trim();
    (!meta.is_empty()).then(|| meta.to_owned())
}

fn find_pair(meta: &str, key: &str) -> Option<(String, std::ops::Range<usize>)> {
    let pair = PAIR.as_ref()?;
    pair.captures_iter(meta).find_map(|captures| {
        let whole = captures.get(0)?;
        if captures.get(1)?.as_str() != key {
            return None;
        }
        let value = (2..=4)
            .find_map(|index| captures.get(index))?
            .as_str()
            .to_owned();
        Some((value, whole.range()))
    })
}
