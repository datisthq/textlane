use textlane_core::ArenaVec;
use textlane_core::mdast::{DefList, DefListDescription, DefListTerm, Node};

use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// Open and close a definition list, its terms and its descriptions.
pub(crate) fn compile_def_list(compiler: &mut Compiler, name: Name, kind: Kind) {
    if kind == Kind::Exit {
        compiler.tail_pop();
        return;
    }
    let children = ArenaVec::new_in(compiler.alloc);
    match name {
        Name::DEF_LIST => compiler.tail_push(Node::DefList(DefList {
            children,
            position: None,
        })),
        Name::DEF_LIST_TERM => compiler.tail_push(Node::DefListTerm(DefListTerm {
            children,
            position: None,
        })),
        _ => {
            let spread = compiler.events[compiler.index..]
                .iter()
                .take_while(|event| event.name != Name::DEF_LIST_DESCRIPTION_MARKER)
                .any(|event| event.name == Name::DEF_LIST_DESCRIPTION_SPREAD);
            compiler.tail_push(Node::DefListDescription(DefListDescription {
                spread,
                children,
                position: None,
            }));
        }
    }
}
