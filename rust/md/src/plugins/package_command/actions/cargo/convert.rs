use crate::helpers::command_table::{BINSTALL_SWITCHES, BINSTALL_VALUE_FLAGS};
use crate::models::options::CargoManager;
use crate::plugins::package_command::actions::args::parse::parse_args;

/// One line of a `cargo` block for a Rust installer: cargo-binstall takes `cargo install`
/// of published crates; every other line, and a flag it lacks, stays as written.
pub(crate) fn convert_cargo(line: &str, manager: CargoManager) -> String {
    match manager {
        CargoManager::Cargo => line.to_owned(),
        CargoManager::CargoBinstall => {
            convert_binstall(line).unwrap_or_else(|| line.to_owned())
        }
    }
}

fn convert_binstall(line: &str) -> Option<String> {
    let args = parse_args(line);
    let mut args = args
        .iter()
        .map(String::as_str)
        .filter(|arg| !arg.is_empty());
    if (args.next(), args.next()) != (Some("cargo"), Some("install")) {
        return None;
    }
    let mut words = vec!["cargo".to_owned(), "binstall".to_owned()];
    let mut crates = 0;
    while let Some(arg) = args.next() {
        let (flag, attached) = match arg.split_once('=') {
            Some((flag, value)) if flag.starts_with("--") => (flag, Some(value)),
            _ => (arg, None),
        };
        if let Some((_, to)) = BINSTALL_VALUE_FLAGS.iter().find(|(from, _)| *from == flag)
        {
            let value = attached.or_else(|| args.next())?;
            words.push(format!("{to} {value}"));
        } else if let Some((_, to)) =
            BINSTALL_SWITCHES.iter().find(|(from, _)| *from == arg)
        {
            words.push((*to).to_owned());
        } else if arg.starts_with('-') {
            return None;
        } else {
            crates += 1;
            words.push(arg.to_owned());
        }
    }
    (crates > 0).then(|| words.join(" "))
}
