use textlane_core::{prefix_base, resolve_url};

use crate::models::options::{MarkdownLinks, PathRewrite};

/// The URL a site serves `url` at, written in the file at `path`; `None` leaves it as it is:
/// a URL with a scheme, protocol-relative or a fragment alone, or a relative one without a
/// file to resolve it against or resolving outside `root`.
pub(crate) fn rewrite_url(
    url: &str,
    path: Option<&str>,
    options: &PathRewrite,
    link: bool,
) -> Option<String> {
    let resolved = resolve_url(url, path, options.root.as_deref())?;
    let file = resolved.file;
    let served = match link && is_markdown(&file) {
        true => match options.pages.get(&file) {
            Some(page) => page.clone(),
            None => page_url(&file, options.markdown_links),
        },
        false if file.is_empty() || !resolved.directory => format!("/{file}"),
        false => format!("/{file}/"),
    };
    Some(format!(
        "{}{}",
        prefix_base(&served, options.base.as_deref()),
        resolved.suffix
    ))
}

fn is_markdown(file: &str) -> bool {
    let name = file.rsplit('/').next().unwrap_or(file);
    name.ends_with(".md") || name.ends_with(".mdx")
}

fn page_url(file: &str, links: MarkdownLinks) -> String {
    let stem = file
        .strip_suffix(".mdx")
        .or_else(|| file.strip_suffix(".md"))
        .unwrap_or(file);
    match links {
        MarkdownLinks::Keep => format!("/{file}"),
        MarkdownLinks::Html => format!("/{stem}.html"),
        MarkdownLinks::Directory => {
            let (parent, name) = match stem.rfind('/') {
                Some(slash) => (&stem[..slash], &stem[slash + 1..]),
                None => ("", stem),
            };
            match (name, parent.is_empty()) {
                ("index", true) => "/".to_owned(),
                ("index", false) => format!("/{parent}/"),
                _ => format!("/{stem}/"),
            }
        }
    }
}
