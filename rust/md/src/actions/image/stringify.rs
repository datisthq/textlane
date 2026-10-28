use textlane_core::mdast::Image;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::safe::SafeConfig;
use crate::utils::serializer::state::Serializer;

/// Write an image, `![alt](url "title")`.
pub(crate) fn stringify_image(state: &mut Serializer, image: &Image) {
    let title = image.title.filter(|title| !title.is_empty());
    state.enter(Construct::Image);
    state.enter(Construct::Label);
    let mut value = "![".to_string();
    let alt = state.safe(
        image.alt.unwrap_or_default(),
        &SafeConfig {
            before: &value,
            after: "]",
            ..SafeConfig::default()
        },
    );
    value.push_str(&alt);
    value.push_str("](");
    state.exit();
    write_destination(state, &mut value, image.url, title.is_some(), ")");
    if let Some(title) = title {
        write_title(state, &mut value, title);
    }
    value.push(')');
    state.exit();
    state.write(&value);
}

/// Append `url` to `value`, enclosed in `<>` when it is empty or has whitespace or controls.
pub(crate) fn write_destination(
    state: &mut Serializer,
    value: &mut String,
    url: &str,
    has_title: bool,
    end: &str,
) {
    if (url.is_empty() && has_title) || has_control_or_space(url) {
        write_literal_destination(state, value, url);
    } else {
        state.enter(Construct::DestinationRaw);
        let after = if has_title { " " } else { end };
        let safe = state.safe(
            url,
            &SafeConfig {
                before: value,
                after,
                ..SafeConfig::default()
            },
        );
        value.push_str(&safe);
        state.exit();
    }
}

pub(crate) fn write_literal_destination(
    state: &mut Serializer,
    value: &mut String,
    url: &str,
) {
    state.enter(Construct::DestinationLiteral);
    value.push('<');
    let safe = state.safe(
        url,
        &SafeConfig {
            before: value,
            after: ">",
            ..SafeConfig::default()
        },
    );
    value.push_str(&safe);
    value.push('>');
    state.exit();
}

/// Append ` "title"` to `value`.
pub(crate) fn write_title(state: &mut Serializer, value: &mut String, title: &str) {
    let quote = state.settings.quote;
    state.enter(if quote == '"' {
        Construct::TitleQuote
    } else {
        Construct::TitleApostrophe
    });
    value.push(' ');
    value.push(quote);
    let after = quote.to_string();
    let safe = state.safe(
        title,
        &SafeConfig {
            before: value,
            after: &after,
            ..SafeConfig::default()
        },
    );
    value.push_str(&safe);
    value.push(quote);
    state.exit();
}

/// Whether `url` has whitespace or a control character, which a raw destination cannot.
pub(crate) fn has_control_or_space(url: &str) -> bool {
    url.chars()
        .any(|character| character <= ' ' || character == '\u{7F}')
}
