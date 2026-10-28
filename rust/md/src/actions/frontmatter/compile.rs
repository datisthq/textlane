use textlane_core::mdast::{Literal, Node, Toml, Yaml};

use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// Frontmatter: a node of its matter's type, valued by what is between its fences.
pub(crate) fn compile_frontmatter(compiler: &mut Compiler, name: Name, kind: Kind) {
    match (name, kind) {
        (Name::FRONTMATTER, Kind::Enter) => {
            let node = create_node(compiler);
            compiler.tail_push(node);
            compiler.buffer();
        }
        (Name::FRONTMATTER, Kind::Exit) => {
            let value = trim_line_endings(compiler.resume());
            match compiler.tail_mut() {
                Node::Yaml(node) => node.value = value,
                Node::Toml(node) => node.value = value,
                Node::Literal(node) => node.value = value,
                _ => {}
            }
            compiler.tail_pop();
        }
        _ => {}
    }
}

// The matter is the first whose fences and place match: the one the tokenizer took
fn create_node<'a>(compiler: &Compiler<'a, '_>) -> Node<'a> {
    let events = compiler.events;
    let line = events[compiler.index].point.line;
    let mut sequences = vec![];
    for (index, event) in events.iter().enumerate().skip(compiler.index + 1) {
        if event.name == Name::FRONTMATTER {
            break;
        }
        if event.name == Name::FRONTMATTER_SEQUENCE && event.kind == Kind::Exit {
            sequences.push(compiler.slice_at(index));
        }
    }
    let (open, close) = match sequences.as_slice() {
        [open, .., close] => (*open, *close),
        _ => ("", ""),
    };
    let kind = compiler
        .frontmatter
        .iter()
        .find(|matter| {
            matter.open == open && matter.close == close && (line == 1 || matter.anywhere)
        })
        .map_or("", |matter| matter.kind.as_str());
    match kind {
        "yaml" => Node::Yaml(Yaml {
            value: "",
            position: None,
        }),
        "toml" => Node::Toml(Toml {
            value: "",
            position: None,
        }),
        _ => Node::Literal(Literal {
            kind: compiler.alloc_str(kind),
            value: "",
            position: None,
        }),
    }
}

// mdast-util-frontmatter drops the line ending after the opening fence and the one
// before the closing fence
fn trim_line_endings(value: &str) -> &str {
    let value = value
        .strip_prefix("\r\n")
        .or_else(|| value.strip_prefix(['\n', '\r']))
        .unwrap_or(value);
    value
        .strip_suffix("\r\n")
        .or_else(|| value.strip_suffix(['\n', '\r']))
        .unwrap_or(value)
}
