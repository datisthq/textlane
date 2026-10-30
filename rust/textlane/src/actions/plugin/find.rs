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
    #[cfg(feature = "schema")]
    pub(crate) upstream: &'static str,
}

macro_rules! plugin {
    ($name:literal, $plugin:path, $upstream:literal, unit) => {
        PluginEntry {
            name: $name,
            build: |_, _| Ok(Box::new($plugin)),
            #[cfg(feature = "schema")]
            options: None,
            #[cfg(feature = "schema")]
            upstream: $upstream,
        }
    };
    ($name:literal, $plugin:ty, $upstream:literal) => {
        PluginEntry {
            name: $name,
            build: |_, options| Ok(Box::new(read_options::<$plugin>(options)?)),
            #[cfg(feature = "schema")]
            options: Some(|generator| generator.subschema_for::<$plugin>()),
            #[cfg(feature = "schema")]
            upstream: $upstream,
        }
    };
}

/// Every public plugin, in the order the bindings list them.
pub(crate) const PLUGINS: &[PluginEntry] = &[
    #[cfg(feature = "md")]
    plugin!("md.parse", textlane_md::Parse, "remark-parse"),
    #[cfg(feature = "md")]
    plugin!("md.directive", textlane_md::Directive, "remark-directive"),
    #[cfg(feature = "md")]
    plugin!("md.gfm", textlane_md::Gfm, "remark-gfm"),
    #[cfg(feature = "md")]
    plugin!(
        "md.definitionList",
        textlane_md::DefinitionList,
        "remark-definition-list",
        unit
    ),
    #[cfg(feature = "md")]
    plugin!("md.math", textlane_md::Math, "remark-math"),
    #[cfg(feature = "md")]
    plugin!("md.mdx", textlane_md::Mdx, "remark-mdx"),
    #[cfg(feature = "md")]
    plugin!(
        "md.frontmatter",
        textlane_md::Frontmatter,
        "remark-frontmatter"
    ),
    #[cfg(feature = "md")]
    plugin!("md.gemoji", textlane_md::Gemoji, "remark-gemoji", unit),
    #[cfg(feature = "md")]
    plugin!("md.component", textlane_md::Component, ""),
    #[cfg(feature = "md")]
    plugin!(
        "md.packageCommand",
        textlane_md::PackageCommand,
        "npm-to-yarn"
    ),
    #[cfg(feature = "md")]
    plugin!("md.headingId", textlane_md::HeadingId, "remark-heading-id"),
    #[cfg(feature = "md")]
    plugin!("md.pathRewrite", textlane_md::PathRewrite, ""),
    #[cfg(feature = "md")]
    plugin!("md.strip", textlane_md::Strip, "strip-markdown"),
    #[cfg(feature = "md")]
    plugin!("md.toc", textlane_md::Toc, "remark-toc"),
    #[cfg(feature = "md")]
    plugin!("md.stringify", textlane_md::Stringify, "remark-stringify"),
    #[cfg(feature = "md")]
    plugin!("md.toHtml", textlane_md::ToHtml, "remark-rehype"),
    #[cfg(feature = "md")]
    plugin!(
        "md.toNaturalLanguage",
        textlane_md::ToNaturalLanguage,
        "remark-retext"
    ),
    #[cfg(feature = "html")]
    plugin!("html.parse", textlane_html::Parse, "rehype-parse"),
    #[cfg(feature = "html")]
    plugin!("html.raw", textlane_html::Raw, "rehype-raw"),
    #[cfg(feature = "html")]
    plugin!("html.format", textlane_html::Format, "rehype-format"),
    #[cfg(feature = "html")]
    plugin!("html.sanitize", textlane_html::Sanitize, "rehype-sanitize"),
    #[cfg(feature = "html")]
    plugin!("html.slug", textlane_html::Slug, "rehype-slug"),
    #[cfg(feature = "html")]
    plugin!(
        "html.externalLinks",
        textlane_html::ExternalLinks,
        "rehype-external-links"
    ),
    #[cfg(feature = "html")]
    plugin!(
        "html.unwrapImages",
        textlane_html::UnwrapImages,
        "rehype-unwrap-images",
        unit
    ),
    #[cfg(feature = "html")]
    plugin!(
        "html.extractToc",
        textlane_html::ExtractToc,
        "@stefanprobst/rehype-extract-toc"
    ),
    #[cfg(feature = "highlight")]
    plugin!(
        "html.highlight",
        textlane_html::Highlight,
        "@shikijs/rehype"
    ),
    #[cfg(feature = "katex")]
    plugin!("html.katex", textlane_html::Katex, "rehype-katex"),
    #[cfg(feature = "html")]
    plugin!(
        "html.toMarkdown",
        textlane_html::ToMarkdown,
        "rehype-remark"
    ),
    #[cfg(feature = "html")]
    plugin!(
        "html.toNaturalLanguage",
        textlane_html::ToNaturalLanguage,
        "rehype-retext",
        unit
    ),
    #[cfg(feature = "html")]
    plugin!(
        "html.stringify",
        textlane_html::Stringify,
        "rehype-stringify"
    ),
    #[cfg(feature = "mdx")]
    plugin!("mdx.compile", textlane_mdx::Compile, "@mdx-js/mdx"),
    #[cfg(feature = "nl")]
    plugin!("nl.parse", textlane_nl::Parse, "retext-english"),
    #[cfg(feature = "nl")]
    plugin!("nl.equality", textlane_nl::Equality, "retext-equality"),
    #[cfg(feature = "nl")]
    plugin!("nl.passive", textlane_nl::Passive, "retext-passive"),
    #[cfg(feature = "nl")]
    plugin!(
        "nl.readability",
        textlane_nl::Readability,
        "retext-readability"
    ),
    #[cfg(feature = "nl")]
    plugin!(
        "nl.repeatedWords",
        textlane_nl::RepeatedWords,
        "retext-repeated-words",
        unit
    ),
    #[cfg(feature = "nl")]
    plugin!(
        "nl.sentenceSpacing",
        textlane_nl::SentenceSpacing,
        "retext-sentence-spacing"
    ),
    #[cfg(feature = "nl")]
    plugin!(
        "nl.stringify",
        textlane_nl::Stringify,
        "retext-stringify",
        unit
    ),
    #[cfg(feature = "nl")]
    plugin!(
        "nl.smartypants",
        textlane_nl::Smartypants,
        "retext-smartypants"
    ),
    #[cfg(feature = "nl")]
    plugin!("nl.quotes", textlane_nl::Quotes, "retext-quotes"),
    #[cfg(feature = "nl")]
    plugin!(
        "nl.contractions",
        textlane_nl::Contractions,
        "retext-contractions"
    ),
    #[cfg(feature = "nl")]
    plugin!(
        "nl.indefiniteArticle",
        textlane_nl::IndefiniteArticle,
        "retext-indefinite-article",
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
