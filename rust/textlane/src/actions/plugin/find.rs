use serde_json::Value;
use textlane_core::{Error, Plugin};

/// Builds a plugin from its name and its options, as JSON.
pub(crate) type BuildPlugin = fn(&str, Value) -> Result<Box<dyn Plugin>, Error>;

/// One public plugin: its name in a pipeline, its builder and, for the bindings' generated
/// types, the schema of its options.
pub(crate) struct PluginEntry {
    pub(crate) name: &'static str,
    pub(crate) build: BuildPlugin,
    #[cfg(feature = "schema")]
    pub(crate) options: Option<fn(&mut schemars::SchemaGenerator) -> schemars::Schema>,
}

macro_rules! plugin {
    ($name:literal, $plugin:path, unit) => {
        PluginEntry {
            name: $name,
            build: |_, _| Ok(Box::new($plugin)),
            #[cfg(feature = "schema")]
            options: None,
        }
    };
    ($name:literal, $plugin:ty) => {
        PluginEntry {
            name: $name,
            build: |_, options| Ok(Box::new(read_options::<$plugin>(options)?)),
            #[cfg(feature = "schema")]
            options: Some(|generator| generator.subschema_for::<$plugin>()),
        }
    };
}

/// Every public plugin, in the order the bindings list them.
pub(crate) const PLUGINS: &[PluginEntry] = &[
    #[cfg(feature = "md")]
    plugin!("md.parse", textlane_md::Parse),
    #[cfg(feature = "md")]
    plugin!("md.directive", textlane_md::Directive),
    #[cfg(feature = "md")]
    plugin!("md.gfm", textlane_md::Gfm),
    #[cfg(feature = "md")]
    plugin!("md.definitionList", textlane_md::DefinitionList, unit),
    #[cfg(feature = "md")]
    plugin!("md.math", textlane_md::Math),
    #[cfg(feature = "md")]
    plugin!("md.mdx", textlane_md::Mdx),
    #[cfg(feature = "md")]
    plugin!("md.frontmatter", textlane_md::Frontmatter),
    #[cfg(feature = "md")]
    plugin!("md.gemoji", textlane_md::Gemoji, unit),
    #[cfg(feature = "md")]
    plugin!("md.stringify", textlane_md::Stringify),
    #[cfg(feature = "md")]
    plugin!("md.toHtml", textlane_md::ToHtml),
    #[cfg(feature = "md")]
    plugin!("md.toNaturalLanguage", textlane_md::ToNaturalLanguage),
    #[cfg(feature = "html")]
    plugin!("html.parse", textlane_html::Parse),
    #[cfg(feature = "html")]
    plugin!("html.raw", textlane_html::Raw),
    #[cfg(feature = "html")]
    plugin!("html.format", textlane_html::Format),
    #[cfg(feature = "html")]
    plugin!("html.sanitize", textlane_html::Sanitize),
    #[cfg(feature = "html")]
    plugin!("html.slug", textlane_html::Slug),
    #[cfg(feature = "highlight")]
    plugin!("html.highlight", textlane_html::Highlight),
    #[cfg(feature = "html")]
    plugin!("html.toMarkdown", textlane_html::ToMarkdown),
    #[cfg(feature = "html")]
    plugin!(
        "html.toNaturalLanguage",
        textlane_html::ToNaturalLanguage,
        unit
    ),
    #[cfg(feature = "html")]
    plugin!("html.stringify", textlane_html::Stringify),
    #[cfg(feature = "mdx")]
    plugin!("mdx.compile", textlane_mdx::Compile),
    #[cfg(feature = "nl")]
    plugin!("nl.parse", textlane_nl::Parse),
    #[cfg(feature = "nl")]
    plugin!("nl.equality", textlane_nl::Equality),
    #[cfg(feature = "nl")]
    plugin!("nl.passive", textlane_nl::Passive),
    #[cfg(feature = "nl")]
    plugin!("nl.readability", textlane_nl::Readability),
    #[cfg(feature = "nl")]
    plugin!("nl.repeatedWords", textlane_nl::RepeatedWords, unit),
    #[cfg(feature = "nl")]
    plugin!("nl.sentenceSpacing", textlane_nl::SentenceSpacing),
    #[cfg(feature = "nl")]
    plugin!("nl.stringify", textlane_nl::Stringify, unit),
    #[cfg(feature = "nl")]
    plugin!("nl.smartypants", textlane_nl::Smartypants),
    #[cfg(feature = "nl")]
    plugin!("nl.quotes", textlane_nl::Quotes),
    #[cfg(feature = "nl")]
    plugin!("nl.contractions", textlane_nl::Contractions),
    #[cfg(feature = "nl")]
    plugin!("nl.indefiniteArticle", textlane_nl::IndefiniteArticle, unit),
];

/// The builder of the plugin a pipeline names, when the crate has one by that name.
pub(crate) fn find_plugin(name: &str) -> Option<BuildPlugin> {
    if let Some(entry) = PLUGINS.iter().find(|entry| entry.name == name) {
        return Some(entry.build);
    }
    #[cfg(feature = "testing")]
    return crate::utils::testing::plugins::find_test_plugin(name);
    #[cfg(not(feature = "testing"))]
    None
}

fn read_options<T: serde::de::DeserializeOwned + Default>(
    options: Value,
) -> Result<T, Error> {
    if options.is_null() {
        return Ok(T::default());
    }
    serde_json::from_value(options)
        .map_err(|error| Error::new("invalid-options", error.to_string()))
}
