use crate::helpers::command::{drop_separator, join_args, map_flags, read_arg, trim_js};
use crate::helpers::command_table::{DENO_ADD_FLAGS, UNCONVERTED};
use crate::plugins::package_command::actions::args::parse::parse_args;

/// npm-to-yarn's `npmToDeno`: one npm command, the text after `npm `, for deno.
pub(crate) fn convert_deno(command: Option<&str>) -> String {
    let mut args = parse_args(trim_js(command.unwrap_or_default()));
    drop_separator(&mut args);
    let first = args.first().cloned().unwrap_or_default();
    let global = args.iter().any(|arg| arg == "-g" || arg == "--global");
    let mut program = "deno";
    match first.as_str() {
        "install" | "i" | "add" => {
            if args.len() == 1 {
                args = vec!["install".to_owned()];
            } else if global {
                args = with_named(&["install", "-g"], &args);
            } else {
                args[0] = "add".to_owned();
                map_flags(&mut args, &DENO_ADD_FLAGS);
            }
        }
        "uninstall" | "un" | "remove" | "r" | "rm" => {
            if global {
                args = with_named(&["uninstall", "--global"], &args);
            } else {
                args[0] = "remove".to_owned();
                args.retain(|arg| arg != "--save-dev" && arg != "-D");
            }
        }
        "ci" | "outdated" => {}
        "update" | "up" => args[0] = "update".to_owned(),
        "run" => args[0] = "task".to_owned(),
        "test" | "t" | "tst" => {
            args[0] = "test".to_owned();
            args.insert(0, "task".to_owned());
        }
        "start" | "stop" => args.insert(0, "task".to_owned()),
        "init" | "create" => {
            if read_arg(&args, 1).is_some_and(|name| !name.starts_with('-')) {
                args[0] = "create".to_owned();
                args.insert(1, "--npm".to_owned());
            } else {
                args[0] = "init".to_owned();
            }
        }
        "exec" => args[0] = "x".to_owned(),
        _ => program = "npm",
    }
    args.retain(|arg| arg != "--");
    let suffix = if program == "npm" { UNCONVERTED } else { "" };
    trim_js(&format!("{program} {}{suffix}", join_args(&args))).to_owned()
}

// The verb, then the arguments after the command that are not flags
fn with_named(verb: &[&str], args: &[String]) -> Vec<String> {
    let named = args
        .iter()
        .skip(1)
        .filter(|arg| !arg.starts_with('-'))
        .cloned();
    verb.iter()
        .map(|word| (*word).to_owned())
        .chain(named)
        .collect()
}
