use textlane_core::mdast::Code;

use crate::utils::serializer::construct::Construct;
use crate::utils::serializer::format::{format_code_as_indented, measure_longest_streak};
use crate::utils::serializer::safe::SafeConfig;
use crate::utils::serializer::sink::Layer;
use crate::utils::serializer::state::Serializer;

/// Write code, fenced or indented.
pub(crate) fn stringify_code(state: &mut Serializer, code: &Code) {
    let raw = code.value;
    if format_code_as_indented(code, state.settings) {
        state.enter(Construct::CodeIndented);
        state.open_layer(Layer::CodeIndented);
        state.write(raw);
        state.close_layer();
        state.exit();
        return;
    }
    let marker = state.settings.fence;
    let (lang_construct, meta_construct) = if marker == '`' {
        (
            Construct::CodeFencedLangGraveAccent,
            Construct::CodeFencedMetaGraveAccent,
        )
    } else {
        (
            Construct::CodeFencedLangTilde,
            Construct::CodeFencedMetaTilde,
        )
    };
    let sequence = marker
        .to_string()
        .repeat((measure_longest_streak(raw, marker) + 1).max(3));
    state.enter(Construct::CodeFenced);
    let mut value = sequence.clone();
    let lang = code.lang.filter(|lang| !lang.is_empty());
    if let Some(lang) = lang {
        state.enter(lang_construct);
        let config = SafeConfig {
            before: &value,
            after: " ",
            encode: &['`'],
            ..SafeConfig::default()
        };
        let safe = state.safe(lang, &config);
        value.push_str(&safe);
        state.exit();
    }
    if let (Some(_), Some(meta)) = (lang, code.meta.filter(|meta| !meta.is_empty())) {
        state.enter(meta_construct);
        value.push(' ');
        let config = SafeConfig {
            before: &value,
            after: "\n",
            encode: &['`'],
            ..SafeConfig::default()
        };
        let safe = state.safe(meta, &config);
        value.push_str(&safe);
        state.exit();
    }
    value.push('\n');
    if !raw.is_empty() {
        value.push_str(raw);
        value.push('\n');
    }
    value.push_str(&sequence);
    state.exit();
    state.write(&value);
}
