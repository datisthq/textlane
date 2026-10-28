use textlane_core::hast::{Element, Node};

use crate::helpers::value::read_value;
use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Kind};

/// `<audio>` and `<video>`: a link to the source around the fallback content, or around
/// the poster as an image; fallback content with links or flow content stays as it is.
pub(crate) fn convert_media<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    element: &'n Element<'a>,
) -> Converted {
    let poster = if element.tag_name == "video" {
        read_value(element, "poster")
    } else {
        Default::default()
    };
    let mut source = read_value(element, "src");
    let mut nodes = state.all(&element.children);
    let has_link = nodes.iter().any(|node| state.store.items[*node].link);
    if has_link || state.store.is_wrap_needed(&nodes) {
        return Converted::Many(nodes);
    }
    for child in &element.children {
        if !source.is_empty() {
            break;
        }
        if let Node::Element(child) = child
            && child.tag_name == "source"
        {
            source = read_value(child, "src");
        }
    }
    if !poster.is_empty() {
        let alt = state.alloc.alloc_str(&state.store.read_string(&nodes));
        let url = state.resolve(Some(&poster));
        let image = Kind::Image {
            url,
            title: None,
            alt,
        };
        nodes = vec![state.store.add(image, vec![], element.position)];
    }
    let title = state.read_optional(element, "title");
    let url = state.resolve(Some(&source));
    Converted::One(
        state
            .store
            .add(Kind::Link { url, title }, nodes, element.position),
    )
}
