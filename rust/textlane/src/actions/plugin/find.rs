use serde_json::Value;
use textlane_core::{Error, Plugin};

/// Builds a plugin from its name and its options, as JSON.
pub(crate) type BuildPlugin = fn(&str, Value) -> Result<Box<dyn Plugin>, Error>;

/// The builder of the plugin a pipeline names, when the crate has one by that name.
pub(crate) fn find_plugin(name: &str) -> Option<BuildPlugin> {
    let build: BuildPlugin = match name {
        #[cfg(feature = "md")]
        "md.parse" => {
            |_, options| Ok(Box::new(textlane_md::parse(read_options(options)?)))
        }
        #[cfg(feature = "md")]
        "md.directive" => {
            |_, options| Ok(Box::new(textlane_md::directive(read_options(options)?)))
        }
        #[cfg(feature = "md")]
        "md.gfm" => |_, options| Ok(Box::new(textlane_md::gfm(read_options(options)?))),
        #[cfg(feature = "md")]
        "md.definitionList" => |_, _| Ok(Box::new(textlane_md::definition_list())),
        #[cfg(feature = "md")]
        "md.math" => |_, options| Ok(Box::new(textlane_md::math(read_options(options)?))),
        #[cfg(feature = "md")]
        "md.frontmatter" => {
            |_, options| Ok(Box::new(textlane_md::frontmatter(read_options(options)?)))
        }
        #[cfg(feature = "md")]
        "md.gemoji" => |_, _| Ok(Box::new(textlane_md::gemoji())),
        #[cfg(feature = "md")]
        "md.stringify" => {
            |_, options| Ok(Box::new(textlane_md::stringify(read_options(options)?)))
        }
        #[cfg(feature = "md")]
        "md.toHtml" => {
            |_, options| Ok(Box::new(textlane_md::to_html(read_options(options)?)))
        }
        #[cfg(feature = "html")]
        "html.parse" => {
            |_, options| Ok(Box::new(textlane_html::parse(read_options(options)?)))
        }
        #[cfg(feature = "html")]
        "html.raw" => {
            |_, options| Ok(Box::new(textlane_html::raw(read_options(options)?)))
        }
        #[cfg(feature = "html")]
        "html.format" => {
            |_, options| Ok(Box::new(textlane_html::format(read_options(options)?)))
        }
        #[cfg(feature = "html")]
        "html.sanitize" => {
            |_, options| Ok(Box::new(textlane_html::sanitize(read_options(options)?)))
        }
        #[cfg(feature = "html")]
        "html.slug" => {
            |_, options| Ok(Box::new(textlane_html::slug(read_options(options)?)))
        }
        #[cfg(feature = "highlight")]
        "html.highlight" => {
            |_, options| Ok(Box::new(textlane_html::highlight(read_options(options)?)))
        }
        #[cfg(feature = "html")]
        "html.toMarkdown" => {
            |_, options| Ok(Box::new(textlane_html::to_markdown(read_options(options)?)))
        }
        #[cfg(feature = "html")]
        "html.stringify" => {
            |_, options| Ok(Box::new(textlane_html::stringify(read_options(options)?)))
        }
        #[cfg(feature = "testing")]
        _ => return crate::utils::testing::plugins::find_test_plugin(name),
        #[cfg(not(feature = "testing"))]
        _ => return None,
    };
    Some(build)
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
