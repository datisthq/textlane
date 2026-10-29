use oxc_syntax::identifier::{is_identifier_part, is_identifier_start};

/// estree-util-is-identifier-name's `name`: a JS identifier, or with `jsx` one that may also
/// hold dashes, as JSX names do.
pub(crate) fn is_identifier_name(value: &str, jsx: bool) -> bool {
    let mut characters = value.chars();
    characters.next().is_some_and(is_identifier_start)
        && characters
            .all(|character| is_identifier_part(character) || (jsx && character == '-'))
}
