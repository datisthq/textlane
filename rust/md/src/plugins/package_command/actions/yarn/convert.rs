use crate::helpers::command::{
    convert_init, drop_separator, join_args, map_flags, read_arg,
    replace_pack_destination, trim_js,
};
use crate::helpers::command_table::{
    UNCHANGED_COMMANDS, UNCONVERTED, VERSION_FLAGS, YARN_COMMANDS, YARN_INSTALL_FLAGS,
};
use crate::plugins::package_command::actions::args::parse::parse_args;

/// npm-to-yarn's `npmToYarn`: one npm command, the text after `npm `, for yarn. `None` is
/// a bare `npm`, which upstream writes back as `npm undefined`.
pub(crate) fn convert_yarn(command: Option<&str>) -> String {
    let mut args = parse_args(trim_js(command.unwrap_or_default()));
    drop_separator(&mut args);
    let first = args.first().cloned().unwrap_or_default();
    if UNCHANGED_COMMANDS.contains(&first.as_str()) {
        return format!("yarn {}", join_args(&args));
    }
    let args = match first.as_str() {
        "install" | "i" => convert_install(args),
        "uninstall" | "un" | "remove" | "r" | "rm" => {
            args[0] = "remove".to_owned();
            convert_install_args(args)
        }
        "version" => {
            map_flags(&mut args, &VERSION_FLAGS);
            args
        }
        "rb" | "rebuild" => {
            args[0] = "add".to_owned();
            args.push("--force".to_owned());
            args
        }
        "run" => convert_run(args),
        "exec" => {
            args[0] = "run".to_owned();
            convert_run(args)
        }
        "ls" | "list" => convert_list(args),
        "init" | "create" => convert_init(args),
        "ln" => rename(args, "link"),
        "t" | "tst" => rename(args, "test"),
        "outdated" => args,
        "pack" => args
            .into_iter()
            .map(|item| replace_pack_destination(&item, "--filename ").unwrap_or(item))
            .collect(),
        _ => {
            return format!("npm {}{UNCONVERTED}", command.unwrap_or("undefined"));
        }
    };
    format!("yarn {}", join_args(&args))
}

fn convert_install(mut args: Vec<String>) -> Vec<String> {
    if args.len() == 1 {
        return vec!["install".to_owned()];
    }
    args[0] = "add".to_owned();
    convert_install_args(args)
}

fn convert_install_args(mut args: Vec<String>) -> Vec<String> {
    if args.iter().any(|arg| arg == "--global" || arg == "-g") {
        args.insert(0, "global".to_owned());
    }
    map_flags(&mut args, &YARN_INSTALL_FLAGS);
    args
}

fn convert_run(mut args: Vec<String>) -> Vec<String> {
    let script = read_arg(&args, 1).filter(|script| {
        !UNCHANGED_COMMANDS.contains(script) && !YARN_COMMANDS.contains(script)
    });
    if script.is_some() {
        args.remove(0);
    }
    args
}

fn convert_list(mut args: Vec<String>) -> Vec<String> {
    args[0] = "list".to_owned();
    let count = args
        .iter()
        .skip(1)
        .take_while(|item| !item.starts_with('-'))
        .count();
    if count > 0 {
        let pattern = format!("\"{}\"", args[1..=count].join("|"));
        args.splice(1..=count, ["--pattern".to_owned(), pattern]);
    }
    args
}

fn rename(mut args: Vec<String>, to: &str) -> Vec<String> {
    args[0] = to.to_owned();
    args
}
