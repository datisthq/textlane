use crate::helpers::command::{
    convert_init, drop_separator, join_args, map_flags, replace_pack_destination, trim_js,
};
use crate::helpers::command_table::{PNPM_INSTALL_FLAGS, PNPM_LIST_FLAGS, UNCONVERTED};
use crate::plugins::package_command::actions::args::parse::parse_args;

/// npm-to-yarn's `npmToPnpm`: one npm command, the text after `npm `, for pnpm.
pub(crate) fn convert_pnpm(command: Option<&str>) -> String {
    let mut args = parse_args(trim_js(command.unwrap_or_default()));
    drop_separator(&mut args);
    let first = args.first().cloned().unwrap_or_default();
    let args = match first.as_str() {
        "install" | "i" => {
            let named = args.iter().filter(|item| !item.starts_with('-')).count();
            if args.len() > 1 && named > 1 {
                args[0] = "add".to_owned();
            }
            map_flags(&mut args, &PNPM_INSTALL_FLAGS);
            args
        }
        "uninstall" | "un" | "remove" | "r" | "rm" => {
            args[0] = "remove".to_owned();
            map_flags(&mut args, &PNPM_INSTALL_FLAGS);
            args
        }
        "rb" | "rebuild" => {
            args[0] = "rebuild".to_owned();
            convert_filter(args)
        }
        "unlink" => convert_filter(args),
        "run" | "exec" | "outdated" | "start" | "link" | "test" => args,
        "ls" | "list" => args
            .into_iter()
            .map(|item| match item.strip_prefix("--depth=") {
                Some(rest) => format!("--depth {}", rest.split('=').next().unwrap_or("")),
                None => item,
            })
            .map(|mut item| {
                map_flags(std::slice::from_mut(&mut item), &PNPM_LIST_FLAGS);
                item
            })
            .collect(),
        "init" | "create" => convert_init(args),
        "ln" => rename(args, "link"),
        "t" | "tst" => rename(args, "test"),
        "pack" => args
            .into_iter()
            .map(|item| {
                replace_pack_destination(&item, "--pack-destination ").unwrap_or(item)
            })
            .collect(),
        _ => {
            return format!("npm {}{UNCONVERTED}", command.unwrap_or("undefined"));
        }
    };
    format!("pnpm {}", join_args(&args))
}

// npm-to-yarn's `convertFilterArg`: the named packages move behind one `--filter`
fn convert_filter(args: Vec<String>) -> Vec<String> {
    if args.len() <= 1 {
        return args;
    }
    let (named, mut kept): (Vec<_>, Vec<_>) = args
        .into_iter()
        .enumerate()
        .partition(|(index, item)| *index != 0 && !item.starts_with('-'));
    if named.is_empty() {
        return kept.into_iter().map(|(_, item)| item).collect();
    }
    let filter: Vec<String> = named.into_iter().map(|(_, item)| item).collect();
    kept.push((0, "--filter".to_owned()));
    kept.push((0, filter.join(" ")));
    kept.into_iter().map(|(_, item)| item).collect()
}

fn rename(mut args: Vec<String>, to: &str) -> Vec<String> {
    args[0] = to.to_owned();
    args
}
