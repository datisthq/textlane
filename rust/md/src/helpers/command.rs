use crate::helpers::script::is_js_whitespace;

/// `value` with JavaScript's `trim()`.
pub(crate) fn trim_js(value: &str) -> &str {
    value.trim_matches(is_js_whitespace)
}

/// `value` with JavaScript's `trimStart()`.
pub(crate) fn trim_js_start(value: &str) -> &str {
    value.trim_start_matches(is_js_whitespace)
}

/// npm-to-yarn's `args.filter(Boolean).join(' ')`.
pub(crate) fn join_args(args: &[String]) -> String {
    let kept: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|arg| !arg.is_empty())
        .collect();
    kept.join(" ")
}

/// Drop the first `--`, as every npm-to-yarn converter does before reading arguments.
pub(crate) fn drop_separator(args: &mut Vec<String>) {
    if let Some(index) = args.iter().position(|arg| arg == "--") {
        args.remove(index);
    }
}

/// An argument as JavaScript tests it: present and not empty.
pub(crate) fn read_arg(args: &[String], index: usize) -> Option<&str> {
    args.get(index)
        .map(String::as_str)
        .filter(|arg| !arg.is_empty())
}

/// Each argument through a flag table; one it does not list stays.
pub(crate) fn map_flags(args: &mut [String], table: &[(&str, &str)]) {
    for arg in args.iter_mut() {
        if let Some((_, to)) = table.iter().find(|(from, _)| from == arg) {
            *arg = (*to).to_owned();
        }
    }
}

/// `args[0]` and npm-to-yarn's `init` for yarn and pnpm: an initializer after `init` makes
/// it `create`, without `@latest`, and `--scope` goes.
pub(crate) fn convert_init(mut args: Vec<String>) -> Vec<String> {
    if let Some(name) = read_arg(&args, 1).filter(|name| !name.starts_with('-')) {
        let name = strip_latest(name).to_owned();
        args[0] = "create".to_owned();
        args[1] = name;
    }
    args.retain(|arg| arg != "--scope");
    args
}

/// `/(.+)@latest/`'s group: what precedes the last `@latest` not at the start.
fn strip_latest(name: &str) -> &str {
    match name.rfind("@latest") {
        Some(at) if at > 0 => &name[..at],
        _ => name,
    }
}

/// `item` with `/^--pack-destination[\s=]/` replaced by `to`.
pub(crate) fn replace_pack_destination(item: &str, to: &str) -> Option<String> {
    let rest = item.strip_prefix("--pack-destination")?;
    let next = rest.chars().next()?;
    (next == '=' || is_js_whitespace(next))
        .then(|| format!("{to}{}", &rest[next.len_utf8()..]))
}
