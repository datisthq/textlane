use textlane_core::hast::{Node, PropertyValue};
use textlane_core::{Allocator, prefix_base, resolve_url};

use crate::helpers::property::find_property;
use crate::helpers::schema::HTML;
use crate::models::options::PathRewrite;

const DEFAULT_ATTRIBUTES: &[&str] = &["href", "src", "srcset", "poster"];

/// Rewrite the URLs in `options.attributes` of every element under `node`, written in the
/// file at `path`, to the root-relative URLs a site serves them at.
pub fn rewrite_paths<'a>(
    node: &mut Node<'a>,
    options: &PathRewrite,
    path: Option<&str>,
    alloc: &'a Allocator,
) {
    let names: Vec<String> = match &options.attributes {
        Some(names) => names.iter().map(|name| read_property(name)).collect(),
        None => DEFAULT_ATTRIBUTES
            .iter()
            .map(|name| read_property(name))
            .collect(),
    };
    let mut stack: Vec<&mut Node<'a>> = vec![node];
    while let Some(node) = stack.pop() {
        let children = match node {
            Node::Root(root) => &mut root.children,
            Node::Element(element) => {
                for property in element.properties.iter_mut() {
                    if !names.iter().any(|name| name == property.name) {
                        continue;
                    }
                    let PropertyValue::String(url) = property.value else {
                        continue;
                    };
                    let rewritten = match property.name {
                        "srcSet" => rewrite_srcset(url, path, options),
                        "href" => rewrite_url(url, path, options, true),
                        _ => rewrite_url(url, path, options, false),
                    };
                    if let Some(rewritten) = rewritten {
                        property.value =
                            PropertyValue::String(alloc.alloc_str(&rewritten));
                    }
                }
                &mut element.children
            }
            _ => continue,
        };
        stack.extend(children.iter_mut());
    }
}

fn read_property(name: &str) -> String {
    find_property(&HTML, name).property.into_owned()
}

fn rewrite_url(
    url: &str,
    path: Option<&str>,
    options: &PathRewrite,
    link: bool,
) -> Option<String> {
    let resolved = resolve_url(url.trim(), path, options.root.as_deref())?;
    let file = resolved.file;
    let served = match (link, options.pages.get(&file)) {
        (true, Some(page)) => page.clone(),
        _ if file.is_empty() || !resolved.directory => format!("/{file}"),
        _ => format!("/{file}/"),
    };
    Some(format!(
        "{}{}",
        prefix_base(&served, options.base.as_deref()),
        resolved.suffix
    ))
}

// Each candidate, `url [descriptor]`, rewritten; `None` when none is
fn rewrite_srcset(
    srcset: &str,
    path: Option<&str>,
    options: &PathRewrite,
) -> Option<String> {
    let mut changed = false;
    let candidates: Vec<String> = (srcset.split(','))
        .map(|candidate| {
            let candidate = candidate.trim();
            let (url, descriptor) = candidate
                .split_once(char::is_whitespace)
                .map_or((candidate, ""), |(url, rest)| (url, rest.trim()));
            match rewrite_url(url, path, options, false) {
                Some(url) => {
                    changed = true;
                    match descriptor.is_empty() {
                        true => url,
                        false => format!("{url} {descriptor}"),
                    }
                }
                None => candidate.to_owned(),
            }
        })
        .collect();
    changed.then(|| candidates.join(", "))
}

#[cfg(test)]
#[path = "path_unit.rs"]
mod unit;
