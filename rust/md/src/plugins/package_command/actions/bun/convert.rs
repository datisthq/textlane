use crate::helpers::command::{drop_separator, join_args, map_flags, read_arg, trim_js};
use crate::helpers::command_table::{BUN_INSTALL_FLAGS, UNCONVERTED};
use crate::plugins::package_command::actions::args::parse::parse_args;

/// npm-to-yarn's `npmToBun`: one npm command, the text after `npm `, for bun.
pub(crate) fn convert_bun(command: Option<&str>) -> String {
    let mut args = parse_args(trim_js(command.unwrap_or_default()));
    drop_separator(&mut args);
    let first = args.first().cloned().unwrap_or_default();
    let mut program = "bun";
    match first.as_str() {
        "install" | "i" => {
            if args.len() == 1 {
                args = vec!["install".to_owned()];
            } else {
                args[0] = "add".to_owned();
            }
            map_flags(&mut args, &BUN_INSTALL_FLAGS);
        }
        "uninstall" | "un" | "remove" | "r" | "rm" => {
            args[0] = "remove".to_owned();
            map_flags(&mut args, &BUN_INSTALL_FLAGS);
        }
        "cache" => {
            if args.get(1).is_some_and(|arg| arg == "clean") {
                let rest = args.split_off(2);
                args = ["pm", "cache", "rm"].map(str::to_owned).to_vec();
                args.extend(rest);
            } else {
                program = "npm";
            }
        }
        "rebuild" | "rb" => {
            args[0] = "add".to_owned();
            args.push("--force".to_owned());
        }
        "run" | "stop" | "start" | "unlink" => {}
        "list" | "ls" => {
            map_flags(&mut args, &BUN_INSTALL_FLAGS);
            args[0] = "ls".to_owned();
            args.insert(0, "pm".to_owned());
        }
        "init" | "create" => {
            if let Some(name) = read_arg(&args, 1).map(str::to_owned) {
                if let Some(scoped) = name.strip_prefix('@') {
                    program = "bunx";
                    args[1] = format!("@{}", scoped.replacen('/', "/create-", 1));
                    args.remove(0);
                } else if !name.starts_with('-') {
                    program = "bunx";
                    args[1] = format!("create-{}", name.replacen("@latest", "", 1));
                    args.remove(0);
                } else {
                    args[0] = "init".to_owned();
                }
            }
        }
        "link" | "ln" => {
            map_flags(&mut args, &BUN_INSTALL_FLAGS);
            args[0] = "link".to_owned();
        }
        "test" | "t" | "tst" => {
            args[0] = "test".to_owned();
            args.insert(0, "run".to_owned());
        }
        "exec" => {
            program = "bunx";
            args.remove(0);
        }
        _ => program = "npm",
    }
    args.retain(|arg| arg != "--");
    let suffix = if program == "npm" { UNCONVERTED } else { "" };
    format!("{program} {}{suffix}", join_args(&args)).replacen('=', " ", 1)
}
