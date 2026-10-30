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
    if url.is_empty() || url.starts_with('#') || url.starts_with("//") || has_scheme(url)
    {
        return None;
    }
    let end = url.find(['?', '#']).unwrap_or(url.len());
    let (target, suffix) = url.split_at(end);
    if target.is_empty() {
        return None;
    }
    let file = match target.strip_prefix('/') {
        Some(absolute) => normalize_path(absolute)?,
        None => {
            let directory = parent_of(&normalize_path(&slashes(path?))?);
            let joined = normalize_path(&format!("{directory}/{target}"))?;
            relative_to(&joined, options.root.as_deref())?
        }
    };
    let served = match link && is_markdown(&file) {
        true => match options.pages.get(&file) {
            Some(page) => page.clone(),
            None => page_url(&file, options.markdown_links),
        },
        false if file.is_empty() || !names_directory(target) => format!("/{file}"),
        false => format!("/{file}/"),
    };
    Some(format!(
        "{}{suffix}",
        prefix_base(&served, options.base.as_deref())
    ))
}

// A scheme, `a+b-c.d:`, as URLs and is-absolute-url read one
fn has_scheme(url: &str) -> bool {
    let Some(colon) = url.find(':') else {
        return false;
    };
    let scheme = &url.as_bytes()[..colon];
    scheme.first().is_some_and(u8::is_ascii_alphabetic)
        && scheme
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || b"+-.".contains(byte))
}

// Normalizing drops a trailing slash, which a server reads as a different URL
fn names_directory(target: &str) -> bool {
    target.ends_with('/') || matches!(target.rsplit('/').next(), Some("." | ".."))
}

fn slashes(path: &str) -> String {
    path.replace('\\', "/")
}

// `.` and `..` resolved, empty segments dropped; `None` when `..` climbs above the start,
// whether the path is absolute or not
fn normalize_path(path: &str) -> Option<String> {
    let absolute = path.starts_with('/');
    let mut segments: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop()?;
            }
            segment => segments.push(segment),
        }
    }
    let joined = segments.join("/");
    Some(if absolute {
        format!("/{joined}")
    } else {
        joined
    })
}

fn parent_of(path: &str) -> String {
    match path.rfind('/') {
        Some(slash) => path[..slash].to_owned(),
        None => String::new(),
    }
}

// The file's path inside `root`, without a leading slash; `None` outside it
fn relative_to(path: &str, root: Option<&str>) -> Option<String> {
    let Some(root) = root else {
        return Some(path.trim_start_matches('/').to_owned());
    };
    let root = normalize_path(&slashes(root))?;
    if root.is_empty() || root == "/" {
        return Some(path.trim_start_matches('/').to_owned());
    }
    let inside = path.strip_prefix(&root)?;
    match inside.strip_prefix('/') {
        Some(inside) => Some(inside.to_owned()),
        None if inside.is_empty() => Some(String::new()),
        None => None,
    }
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

// `base` once before a root-relative URL: never twice, nothing for `/` or none
fn prefix_base(url: &str, base: Option<&str>) -> String {
    let base = base
        .map(|base| base.trim_end_matches('/'))
        .unwrap_or_default();
    if base.is_empty() || url == base || url.starts_with(&format!("{base}/")) {
        return url.to_owned();
    }
    format!("{base}{url}")
}
