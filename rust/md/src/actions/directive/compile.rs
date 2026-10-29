use textlane_core::ArenaVec;
use textlane_core::mdast::{
    ContainerDirective, LeafDirective, Node, Paragraph, TextDirective,
};

use crate::helpers::attribute::decode_attribute;
use crate::models::directive::{
    CONTAINER, DIRECTIVE_CONTAINER, DIRECTIVE_LEAF, DIRECTIVE_TEXT, LEAF, TEXT,
};
use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// A directive: its name, attributes and, for a container, its label as a paragraph.
pub(crate) fn compile_directive(compiler: &mut Compiler, name: Name, kind: Kind) {
    let names = [&TEXT, &LEAF, &CONTAINER].into_iter().find(|names| {
        name.text()
            .starts_with(names.name.text().trim_end_matches("Name"))
    });
    match (kind, names) {
        (Kind::Enter, _) if name == DIRECTIVE_TEXT => {
            let children = ArenaVec::new_in(compiler.alloc);
            compiler.tail_push(Node::TextDirective(TextDirective {
                name: "",
                attributes: ArenaVec::new_in(compiler.alloc),
                children,
                position: None,
                data: None,
            }));
        }
        (Kind::Enter, _) if name == DIRECTIVE_LEAF => {
            let children = ArenaVec::new_in(compiler.alloc);
            compiler.tail_push(Node::LeafDirective(LeafDirective {
                name: "",
                attributes: ArenaVec::new_in(compiler.alloc),
                children,
                position: None,
                data: None,
            }));
        }
        (Kind::Enter, _) if name == DIRECTIVE_CONTAINER => {
            let children = ArenaVec::new_in(compiler.alloc);
            compiler.tail_push(Node::ContainerDirective(ContainerDirective {
                name: "",
                attributes: ArenaVec::new_in(compiler.alloc),
                label: false,
                children,
                position: None,
                data: None,
            }));
        }
        (Kind::Exit, _)
            if name == DIRECTIVE_TEXT
                || name == DIRECTIVE_LEAF
                || name == DIRECTIVE_CONTAINER =>
        {
            compiler.tail_pop();
        }
        (Kind::Enter, _) if name == CONTAINER.label => {
            if let Node::ContainerDirective(directive) = compiler.tail_mut() {
                directive.label = true;
            }
            compiler.tail_push(Node::Paragraph(Paragraph {
                children: ArenaVec::new_in(compiler.alloc),
                position: None,
                data: None,
            }));
        }
        (Kind::Exit, _) if name == CONTAINER.label => compiler.tail_pop(),
        (Kind::Enter, Some(names)) if name == names.attributes => {
            compiler.directive_attributes.clear();
            // Line endings between attributes would otherwise become text
            compiler.buffer();
        }
        (Kind::Exit, Some(names)) if name == names.attributes => {
            compiler.resume_node();
            let list = std::mem::take(&mut compiler.directive_attributes);
            let mut cleaned: Vec<(&str, String)> = Vec::with_capacity(list.len());
            for (key, value) in list {
                match cleaned.iter_mut().find(|(existing, _)| *existing == key) {
                    Some((_, existing)) if key == "class" => {
                        existing.push(' ');
                        existing.push_str(&value);
                    }
                    Some((_, existing)) => *existing = value,
                    None => cleaned.push((key, value)),
                }
            }
            let mut attributes =
                ArenaVec::with_capacity_in(cleaned.len(), compiler.alloc);
            for (key, value) in cleaned {
                attributes.push((key, compiler.alloc_str(&value)));
            }
            match compiler.tail_mut() {
                Node::TextDirective(node) => node.attributes = attributes,
                Node::LeafDirective(node) => node.attributes = attributes,
                Node::ContainerDirective(node) => node.attributes = attributes,
                _ => {}
            }
        }
        (Kind::Exit, Some(names)) if name == names.name => {
            let value = compiler.slice();
            match compiler.tail_mut() {
                Node::TextDirective(node) => node.name = value,
                Node::LeafDirective(node) => node.name = value,
                Node::ContainerDirective(node) => node.name = value,
                _ => {}
            }
        }
        (Kind::Exit, Some(names)) if name == names.attribute_id_value => {
            let value = decode_attribute(compiler.slice());
            compiler.directive_attributes.push(("id", value));
        }
        (Kind::Exit, Some(names)) if name == names.attribute_class_value => {
            let value = decode_attribute(compiler.slice());
            compiler.directive_attributes.push(("class", value));
        }
        // Attribute names cannot hold character references
        (Kind::Exit, Some(names)) if name == names.attribute_name => {
            let key = compiler.slice();
            compiler.directive_attributes.push((key, String::new()));
        }
        (Kind::Exit, Some(names)) if name == names.attribute_value => {
            let value = decode_attribute(compiler.slice());
            if let Some(last) = compiler.directive_attributes.last_mut() {
                last.1 = value;
            }
        }
        _ => {}
    }
}
