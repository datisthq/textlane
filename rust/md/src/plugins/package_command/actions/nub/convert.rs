use crate::helpers::command::{drop_separator, join_args, map_flags, read_arg, trim_js};
use crate::helpers::command_table::{NUB_INSTALL_FLAGS, UNCONVERTED};
use crate::plugins::package_command::actions::args::parse::parse_args;

/// npm-to-yarn's `npmToNub`: one npm command, the text after `npm `, for nub.
pub(crate) fn convert_nub(command: Option<&str>) -> String {
    let command = trim_js(command.unwrap_or_default());
    let mut args = parse_args(command);
    if args.iter().any(|arg| arg == "--no-package-lock") {
        return format!("npm {command}{UNCONVERTED}");
    }
    drop_separator(&mut args);
    let first = args.first().cloned().unwrap_or_default();
    let mut program = "nub";
    match first.as_str() {
        "install" | "i" => {
            if args.iter().filter(|item| !item.starts_with('-')).count() > 1 {
                args[0] = "add".to_owned();
            }
            map_flags(&mut args, &NUB_INSTALL_FLAGS);
        }
        "uninstall" | "un" | "remove" | "r" | "rm" => {
            args[0] = "remove".to_owned();
            map_flags(&mut args, &NUB_INSTALL_FLAGS);
        }
        "ci" | "dedupe" | "prune" | "outdated" | "ls" | "list" | "run" | "unlink"
        | "pack" => {}
        "update" | "up" => args[0] = "update".to_owned(),
        "rebuild" | "rb" => args[0] = "rebuild".to_owned(),
        "run-script" => args[0] = "run".to_owned(),
        "test" | "t" | "tst" => {
            args[0] = "test".to_owned();
            args.insert(0, "run".to_owned());
        }
        "start" | "stop" => args.insert(0, "run".to_owned()),
        "init" | "create" => match read_arg(&args, 1).map(str::to_owned) {
            Some(name) if name.starts_with('@') => {
                program = "nubx";
                args[1] = name.replacen('/', "/create-", 1);
                args.remove(0);
            }
            Some(name) if !name.starts_with('-') => {
                program = "nubx";
                args[1] = format!("create-{}", name.replacen("@latest", "", 1));
                args.remove(0);
            }
            _ => args[0] = "init".to_owned(),
        },
        "exec" => {
            program = "nubx";
            args.remove(0);
        }
        "link" | "ln" => args[0] = "link".to_owned(),
        _ => program = "npm",
    }
    args.retain(|arg| arg != "--");
    let suffix = if program == "npm" { UNCONVERTED } else { "" };
    trim_js(&format!("{program} {}{suffix}", join_args(&args))).to_owned()
}
