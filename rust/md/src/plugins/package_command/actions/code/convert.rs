use textlane_core::mdast::{Code, Node};
use textlane_core::{Allocator, ArenaVec};

use crate::models::options::PackageCommand;
use crate::plugins::component::actions::element::create::create_element;
use crate::plugins::package_command::actions::cargo::convert::convert_cargo;
use crate::plugins::package_command::actions::npm::convert::convert_npm;
use crate::plugins::package_command::actions::pip::convert::convert_pip;

/// A code block whose language names an ecosystem as the component, one attribute per
/// manager holding the block converted line by line; any other block stays.
pub(crate) fn convert_code<'a>(
    options: &PackageCommand,
    code: Code<'a>,
    alloc: &'a Allocator,
) -> Node<'a> {
    let ecosystems = &options.ecosystems;
    let value = code.value;
    let attributes: Vec<(String, String)> = match code.lang {
        Some("npm") => ecosystems
            .npm
            .iter()
            .map(|&manager| {
                let lines = convert_lines(value, |line| convert_npm(line, manager));
                (manager.name().to_owned(), lines)
            })
            .collect(),
        Some("pip") => ecosystems
            .pip
            .iter()
            .map(|&manager| {
                let lines = convert_lines(value, |line| convert_pip(line, manager));
                (manager.name().to_owned(), lines)
            })
            .collect(),
        Some("cargo") => ecosystems
            .cargo
            .iter()
            .map(|&manager| {
                let lines = convert_lines(value, |line| convert_cargo(line, manager));
                (manager.name().to_owned(), lines)
            })
            .collect(),
        _ => Vec::new(),
    };
    if attributes.is_empty() {
        return Node::Code(code);
    }
    create_element(
        alloc.alloc_str(&options.component),
        attributes,
        ArenaVec::new_in(alloc),
        code.position,
        false,
        alloc,
    )
}

fn convert_lines(value: &str, convert: impl Fn(&str) -> String) -> String {
    let lines: Vec<String> = value.split('\n').map(convert).collect();
    lines.join("\n")
}

#[cfg(test)]
#[path = "convert_unit.rs"]
mod unit;
