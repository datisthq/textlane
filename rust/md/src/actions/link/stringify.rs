use textlane_core::Error;
use textlane_core::mdast::Link;

use crate::actions::image::stringify::{
    has_control_or_space, write_literal_destination, write_title,
};
use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::format::format_link_as_autolink;
use crate::utils::serializer::safe::SafeConfig;
use crate::utils::serializer::state::{Info, Parent, Serializer};

/// Write a link, as an autolink where it can be one.
pub(crate) fn stringify_link(
    state: &mut Serializer,
    link: &Link,
    node: Parent,
    info: Info,
) -> Result<(), Error> {
    if format_link_as_autolink(link, state.settings) {
        let text = state.without_phrasing(|state| {
            state.enter(Construct::Autolink);
            let text = state.container_phrasing(
                node,
                Info {
                    before: "<",
                    after: ">",
                },
            );
            state.exit();
            text
        })?;
        state.write(&format!("<{text}>"));
        return Ok(());
    }
    state.enter(Construct::Link);
    state.enter(Construct::Label);
    let text = state.container_phrasing(
        node,
        Info {
            before: "[",
            after: "](",
        },
    );
    state.exit();
    let text = match text {
        Ok(text) => text,
        Err(error) => {
            state.exit();
            return Err(error);
        }
    };
    let mut value = format!("[{text}](");
    let url = link.url;
    let enclose = has_control_or_space(url);
    let mut title = link.title.filter(|title| !title.is_empty());
    // Text like `a]:` at the start of a line would read as a definition without a title
    if title.is_none() && !enclose && info.before.ends_with(['\n', '\r']) {
        let bytes = text.as_bytes();
        let mut index = 0;
        while index < bytes.len() && bytes[index] != b']' {
            index += if bytes[index] == b'\\' { 2 } else { 1 };
        }
        if bytes.get(index + 1) == Some(&b':') {
            title = Some("");
        }
    }
    if (url.is_empty() && title.is_some()) || enclose {
        write_literal_destination(state, &mut value, url);
    } else {
        state.enter(Construct::DestinationRaw);
        let after = if title.is_some() { " " } else { ")" };
        let safe = state.safe(
            url,
            &SafeConfig {
                before: &value,
                after,
                ..SafeConfig::default()
            },
        );
        value.push_str(&safe);
        state.exit();
    }
    if let Some(title) = title {
        write_title(state, &mut value, title);
    }
    value.push(')');
    state.exit();
    state.write(&value);
    Ok(())
}
