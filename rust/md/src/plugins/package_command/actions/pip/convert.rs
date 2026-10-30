use crate::helpers::command_table::{
    PIP_ADD, PIP_DEV_FLAG, PIP_EDITABLE_FLAG, PIP_FREEZE, PIP_GROUP_FLAG,
    PIP_GROUP_INSTALL, PIP_LIST, PIP_PRE_FLAG, PIP_PROJECT, PIP_REMOVE, PIP_REQUIREMENTS,
    PIP_RUN, PIP_SHOW, PIP_UPGRADE_FLAG, PipEntry, TOOL_INSTALL, TOOL_RUN,
    TOOL_UNINSTALL,
};
use crate::models::options::PipManager;
use crate::plugins::package_command::actions::args::parse::parse_args;

/// One line of a `pip` block for a Python manager, by textlane's own tables; a line the
/// manager has no equivalent for, or pip itself, stays as written.
pub(crate) fn convert_pip(line: &str, manager: PipManager) -> String {
    let column = match manager {
        PipManager::Pip => 0,
        PipManager::Uv => 1,
        PipManager::Poetry => 2,
        PipManager::Pdm => 3,
    };
    let args = parse_args(line);
    let args: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|arg| !arg.is_empty())
        .collect();
    read_line(&args, column).unwrap_or_else(|| line.to_owned())
}

enum Item<'a> {
    Package(&'a str),
    Editable(&'a str),
    Requirement(&'a str),
    Upgrade,
    Pre,
}

fn read_line(args: &[&str], column: usize) -> Option<String> {
    let entry = |entry: PipEntry| entry[column];
    let write = |prefix: &str, rest: &[&str]| match rest.is_empty() {
        true => prefix.to_owned(),
        false => format!("{prefix} {}", rest.join(" ")),
    };
    match args {
        ["pip" | "pip3", rest @ ..] | ["python" | "python3", "-m", "pip", rest @ ..] => {
            read_pip(rest, column)
        }
        ["pipx" | "uvx", ..] | ["uv", "tool", ..] => {
            let (table, rest) = match args {
                ["pipx", "run", rest @ ..] | ["uvx", rest @ ..] => (TOOL_RUN, rest),
                ["pipx", "install", rest @ ..] | ["uv", "tool", "install", rest @ ..] => {
                    (TOOL_INSTALL, rest)
                }
                ["pipx", "uninstall", rest @ ..]
                | ["uv", "tool", "uninstall", rest @ ..] => (TOOL_UNINSTALL, rest),
                _ => return None,
            };
            let tool = rest.first().filter(|tool| !tool.starts_with('-'))?;
            Some(write(entry(table)?, &[&[*tool], &rest[1..]].concat()))
        }
        ["python" | "python3", ..] => Some(write(entry(PIP_RUN)?, args)),
        _ => None,
    }
}

fn read_pip(args: &[&str], column: usize) -> Option<String> {
    let entry = |entry: PipEntry| entry[column];
    match args {
        ["install", rest @ ..] => read_install(rest, column),
        ["uninstall", rest @ ..] => {
            let mut packages = Vec::new();
            for arg in rest {
                match *arg {
                    "-y" | "--yes" => {}
                    flag if flag.starts_with('-') => return None,
                    package => packages.push(package),
                }
            }
            (!packages.is_empty())
                .then_some(())
                .and(entry(PIP_REMOVE))
                .map(|verb| format!("{verb} {}", packages.join(" ")))
        }
        ["list"] => entry(PIP_LIST).map(str::to_owned),
        ["freeze"] => entry(PIP_FREEZE).map(str::to_owned),
        ["show", package] if !package.starts_with('-') => {
            entry(PIP_SHOW).map(|verb| format!("{verb} {package}"))
        }
        _ => None,
    }
}

fn read_install(args: &[&str], column: usize) -> Option<String> {
    let entry = |entry: PipEntry| entry[column];
    let mut items = Vec::new();
    let mut group = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        let (flag, attached) = match arg.split_once('=') {
            Some((flag, value)) if flag.starts_with("--") => (flag, Some(value)),
            _ => (*arg, None),
        };
        let mut value = || attached.or_else(|| rest.next().copied());
        match flag {
            "-r" | "--requirement" => items.push(Item::Requirement(value()?)),
            "-e" | "--editable" => items.push(Item::Editable(value()?)),
            "--group" if group.is_none() => group = Some(value()?),
            "-U" | "--upgrade" if attached.is_none() => items.push(Item::Upgrade),
            "--pre" if attached.is_none() => items.push(Item::Pre),
            flag if flag.starts_with('-') => return None,
            _ => items.push(Item::Package(arg)),
        }
    }
    let named: Vec<&str> = items
        .iter()
        .filter_map(|item| match item {
            Item::Package(name) | Item::Editable(name) => Some(*name),
            _ => None,
        })
        .collect();
    let requirements = items
        .iter()
        .any(|item| matches!(item, Item::Requirement(_)));
    let mut words = Vec::new();
    if requirements {
        if !named.is_empty() || group.is_some() {
            return None;
        }
        words.push(entry(PIP_REQUIREMENTS)?.to_owned());
    } else if named == ["."] {
        if items.len() > 1 || group.is_some() {
            return None;
        }
        return entry(PIP_PROJECT).map(str::to_owned);
    } else if named.is_empty() {
        if items.is_empty() {
            return group.and_then(|group| {
                entry(PIP_GROUP_INSTALL).map(|verb| format!("{verb} {group}"))
            });
        }
        return None;
    } else {
        if named.contains(&".") {
            return None;
        }
        words.push(entry(PIP_ADD)?.to_owned());
        match group {
            Some("dev") => words.push(entry(PIP_DEV_FLAG)?.to_owned()),
            Some(group) => words.push(format!("{} {group}", entry(PIP_GROUP_FLAG)?)),
            None => {}
        }
    }
    for item in &items {
        words.push(match item {
            Item::Package(name) => (*name).to_owned(),
            Item::Editable(path) => format!("{} {path}", entry(PIP_EDITABLE_FLAG)?),
            Item::Requirement(file) => format!("-r {file}"),
            Item::Upgrade => entry(PIP_UPGRADE_FLAG)?.to_owned(),
            Item::Pre => entry(PIP_PRE_FLAG)?.to_owned(),
        });
    }
    Some(words.join(" "))
}

#[cfg(test)]
#[path = "convert_unit.rs"]
mod unit;
