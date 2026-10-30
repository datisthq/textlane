/// A word as nlcst-normalize compares it: lowercase, without dashes and, unless allowed,
/// apostrophes, with `’` read as `'`.
pub(crate) fn normalize_word(value: &str, allow_apostrophes: bool) -> String {
    value
        .to_lowercase()
        .chars()
        .filter_map(|character| match character {
            '-' => None,
            '\'' | '’' if !allow_apostrophes => None,
            '’' => Some('\''),
            character => Some(character),
        })
        .collect()
}
