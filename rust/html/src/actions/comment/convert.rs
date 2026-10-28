use textlane_core::hast::Comment;

use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// A comment: HTML.
pub(crate) fn convert_comment<'a>(
    state: &mut Converter<'a, '_, '_>,
    comment: &Comment<'a>,
) -> Converted {
    let value = state.alloc.alloc_str(&format!("<!--{}-->", comment.value));
    Converted::One(state.store.add(Kind::Html(value), vec![], comment.position))
}
