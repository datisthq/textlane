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
    ($name:literal, $build:path, $options:ty) => {
        PluginEntry {
            name: $name,
            build: |_, options| Ok(Box::new($build(read_options::<$options>(options)?))),
            #[cfg(feature = "schema")]
            options: Some(|generator| generator.subschema_for::<$options>()),
        }
    };
    ($name:literal, $build:path) => {
        PluginEntry {
            name: $name,
            build: |_, _| Ok(Box::new($build())),
            #[cfg(feature = "schema")]
            options: None,
        }
    };
}

/// Every public plugin, in the order the bindings list them.
pub(crate) const PLUGINS: &[PluginEntry] = &[
    #[cfg(feature = "md")]
    plugin!("md.parse", textlane_md::parse, textlane_md::ParseOptions),
    #[cfg(feature = "md")]
    plugin!(
        "md.directive",
        textlane_md::directive,
        textlane_md::DirectiveOptions
    ),
    #[cfg(feature = "md")]
    plugin!("md.gfm", textlane_md::gfm, textlane_md::GfmOptions),
    #[cfg(feature = "md")]
    plugin!("md.definitionList", textlane_md::definition_list),
    #[cfg(feature = "md")]
    plugin!("md.math", textlane_md::math, textlane_md::MathOptions),
    #[cfg(feature = "md")]
    plugin!("md.mdx", textlane_md::mdx, textlane_md::MdxOptions),
    #[cfg(feature = "md")]
    plugin!(
        "md.frontmatter",
        textlane_md::frontmatter,
        textlane_md::FrontmatterOptions
    ),
    #[cfg(feature = "md")]
    plugin!("md.gemoji", textlane_md::gemoji),
    #[cfg(feature = "md")]
    plugin!(
        "md.stringify",
        textlane_md::stringify,
        textlane_md::StringifyOptions
    ),
    #[cfg(feature = "md")]
    plugin!(
        "md.toHtml",
        textlane_md::to_html,
        textlane_md::ToHtmlOptions
    ),
    #[cfg(feature = "md")]
    plugin!(
        "md.toProse",
        textlane_md::to_prose,
        textlane_md::ToProseOptions
    ),
    #[cfg(feature = "html")]
    plugin!(
        "html.parse",
        textlane_html::parse,
        textlane_html::ParseOptions
    ),
    #[cfg(feature = "html")]
    plugin!("html.raw", textlane_html::raw, textlane_html::RawOptions),
    #[cfg(feature = "html")]
    plugin!(
        "html.format",
        textlane_html::format,
        textlane_html::FormatOptions
    ),
    #[cfg(feature = "html")]
    plugin!(
        "html.sanitize",
        textlane_html::sanitize,
        textlane_html::SanitizeOptions
    ),
    #[cfg(feature = "html")]
    plugin!("html.slug", textlane_html::slug, textlane_html::SlugOptions),
    #[cfg(feature = "highlight")]
    plugin!(
        "html.highlight",
        textlane_html::highlight,
        textlane_html::HighlightOptions
    ),
    #[cfg(feature = "html")]
    plugin!(
        "html.toMarkdown",
        textlane_html::to_markdown,
        textlane_html::ToMarkdownOptions
    ),
    #[cfg(feature = "html")]
    plugin!("html.toProse", textlane_html::to_prose),
    #[cfg(feature = "html")]
    plugin!(
        "html.stringify",
        textlane_html::stringify,
        textlane_html::StringifyOptions
    ),
    #[cfg(feature = "mdx")]
    plugin!(
        "mdx.compile",
        textlane_mdx::compile,
        textlane_mdx::CompileOptions
    ),
    #[cfg(feature = "prose")]
    plugin!(
        "prose.parse",
        textlane_prose::parse,
        textlane_prose::ParseOptions
    ),
    #[cfg(feature = "prose")]
    plugin!(
        "prose.equality",
        textlane_prose::equality,
        textlane_prose::EqualityOptions
    ),
    #[cfg(feature = "prose")]
    plugin!(
        "prose.passive",
        textlane_prose::passive,
        textlane_prose::PassiveOptions
    ),
    #[cfg(feature = "prose")]
    plugin!(
        "prose.readability",
        textlane_prose::readability,
        textlane_prose::ReadabilityOptions
    ),
    #[cfg(feature = "prose")]
    plugin!("prose.repeatedWords", textlane_prose::repeated_words),
    #[cfg(feature = "prose")]
    plugin!(
        "prose.sentenceSpacing",
        textlane_prose::sentence_spacing,
        textlane_prose::SentenceSpacingOptions
    ),
    #[cfg(feature = "prose")]
    plugin!("prose.stringify", textlane_prose::stringify),
    #[cfg(feature = "prose")]
    plugin!(
        "prose.smartypants",
        textlane_prose::smartypants,
        textlane_prose::SmartypantsOptions
    ),
    #[cfg(feature = "prose")]
    plugin!(
        "prose.quotes",
        textlane_prose::quotes,
        textlane_prose::QuotesOptions
    ),
    #[cfg(feature = "prose")]
    plugin!(
        "prose.contractions",
        textlane_prose::contractions,
        textlane_prose::ContractionsOptions
    ),
    #[cfg(feature = "prose")]
    plugin!(
        "prose.indefiniteArticle",
        textlane_prose::indefinite_article
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
