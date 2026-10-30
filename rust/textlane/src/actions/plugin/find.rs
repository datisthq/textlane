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
    plugin!("md.toProse", textlane_md::ToProse),
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
    plugin!("html.toProse", textlane_html::ToProse, unit),
    #[cfg(feature = "html")]
    plugin!("html.stringify", textlane_html::Stringify),
    #[cfg(feature = "mdx")]
    plugin!("mdx.compile", textlane_mdx::Compile),
    #[cfg(feature = "prose")]
    plugin!("prose.parse", textlane_prose::Parse),
    #[cfg(feature = "prose")]
    plugin!("prose.equality", textlane_prose::Equality),
    #[cfg(feature = "prose")]
    plugin!("prose.passive", textlane_prose::Passive),
    #[cfg(feature = "prose")]
    plugin!("prose.readability", textlane_prose::Readability),
    #[cfg(feature = "prose")]
    plugin!("prose.repeatedWords", textlane_prose::RepeatedWords, unit),
    #[cfg(feature = "prose")]
    plugin!("prose.sentenceSpacing", textlane_prose::SentenceSpacing),
    #[cfg(feature = "prose")]
    plugin!("prose.stringify", textlane_prose::Stringify, unit),
    #[cfg(feature = "prose")]
    plugin!("prose.smartypants", textlane_prose::Smartypants),
    #[cfg(feature = "prose")]
    plugin!("prose.quotes", textlane_prose::Quotes),
    #[cfg(feature = "prose")]
    plugin!("prose.contractions", textlane_prose::Contractions),
    #[cfg(feature = "prose")]
    plugin!(
        "prose.indefiniteArticle",
        textlane_prose::IndefiniteArticle,
        unit
    ),
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
