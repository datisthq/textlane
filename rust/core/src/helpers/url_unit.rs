use super::{prefix_base, resolve_url};

#[test]
fn resolves_relative_and_root_relative_urls() {
    let resolved = resolve_url(
        "../img/a.png?v=1#x",
        Some("docs/guides/a.html"),
        Some("docs"),
    );
    assert_eq!(
        resolved.map(|url| (url.file, url.suffix)),
        Some(("img/a.png".into(), "?v=1#x"))
    );
    assert_eq!(
        resolve_url("/a/", None, None).map(|url| url.directory),
        Some(true)
    );
}

#[test]
fn leaves_other_hosts_and_backslashes_alone() {
    for url in [
        "https://x.dev",
        "//x.dev",
        "#top",
        "\\\\evil.dev",
        "/\\evil.dev",
        "a\\b",
    ] {
        assert_eq!(resolve_url(url, Some("index.html"), None), None, "{url}");
    }
}

#[test]
fn prefixes_base_once() {
    assert_eq!(prefix_base("/a", Some("/repo/")), "/repo/a");
    assert_eq!(prefix_base("/repo/a", Some("/repo")), "/repo/a");
}
