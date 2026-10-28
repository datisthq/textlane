use textlane_core::ArenaVec;
use textlane_core::hast;
use textlane_core::mdast::DefList;

use crate::utils::converter::state::Converter;

/// A definition list becomes `dl`, its terms and descriptions on lines of their own.
pub(crate) fn convert_def_list<'a>(
    converter: &mut Converter<'a, '_>,
    node: &DefList<'a>,
    results: ArenaVec<'a, hast::Node<'a>>,
) -> hast::Node<'a> {
    let any = !results.is_empty();
    let mut children = ArenaVec::with_capacity_in(results.len() * 2 + 1, converter.alloc);
    for child in results {
        children.push(converter.text("\n"));
        children.push(child);
    }
    if any {
        children.push(converter.text("\n"));
    }
    converter.element("dl", children, node.position)
}
