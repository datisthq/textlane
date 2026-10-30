use crate::helpers::command::trim_js_start;
use crate::helpers::command_table::EXECUTORS;
use crate::models::options::NpmManager;
use crate::plugins::package_command::actions::bun::convert::convert_bun;
use crate::plugins::package_command::actions::deno::convert::convert_deno;
use crate::plugins::package_command::actions::nub::convert::convert_nub;
use crate::plugins::package_command::actions::pnpm::convert::convert_pnpm;
use crate::plugins::package_command::actions::yarn::convert::convert_yarn;

/// npm-to-yarn's `convert(line, manager)`, except that npm keeps a line no executor leads:
/// upstream reads it as yarn's, so `npm i yarn` would become `npm i npm install`.
pub(crate) fn convert_npm(line: &str, manager: NpmManager) -> String {
    let trimmed = trim_js_start(line);
    let executor = EXECUTORS.into_iter().find(|executor| {
        trimmed
            .strip_prefix(executor)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(' '))
    });
    if let Some(executor) = executor {
        return line.replacen(executor, read_executor(manager), 1);
    }
    let convert: fn(Option<&str>) -> String = match manager {
        NpmManager::Npm => return line.to_owned(),
        NpmManager::Pnpm => convert_pnpm,
        NpmManager::Yarn => convert_yarn,
        NpmManager::Bun => convert_bun,
        NpmManager::Deno => convert_deno,
        NpmManager::Nub => convert_nub,
    };
    replace_commands(line, convert)
}

fn read_executor(manager: NpmManager) -> &'static str {
    match manager {
        NpmManager::Npm => "npx",
        NpmManager::Yarn => "yarn dlx",
        NpmManager::Pnpm => "pnpm dlx",
        NpmManager::Bun => "bun x",
        NpmManager::Deno => "deno x",
        NpmManager::Nub => "nubx",
    }
}

// `line.replace(/npm(?: +([^&\n\r]*))?/gm, convert)`: `npm` matches inside words too, as
// in `pnpm`, and the group is absent, not empty, when no space follows
fn replace_commands(line: &str, convert: fn(Option<&str>) -> String) -> String {
    let mut written = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(at) = rest.find("npm") {
        written.push_str(&rest[..at]);
        let after = &rest[at + 3..];
        let tail = after.trim_start_matches(' ');
        if tail.len() == after.len() {
            written.push_str(&convert(None));
            rest = after;
            continue;
        }
        let end = tail.find(['&', '\n', '\r']).unwrap_or(tail.len());
        written.push_str(&convert(Some(&tail[..end])));
        rest = &tail[end..];
    }
    written.push_str(rest);
    written
}

#[cfg(test)]
#[path = "convert_unit.rs"]
mod unit;
