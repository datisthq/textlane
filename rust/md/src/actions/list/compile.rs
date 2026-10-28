use textlane_core::ArenaVec;
use textlane_core::mdast::{List, Node};

use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// Open and close a list; the first item's number is where an ordered list starts.
pub(crate) fn compile_list(compiler: &mut Compiler, name: Name, kind: Kind) {
    match (name, kind) {
        (Name::LIST_ITEM_VALUE, Kind::Exit) => {
            let start = compiler.slice().parse().ok();
            if let Some(Node::List(list)) = compiler.penultimate_mut()
                && list.start.is_none()
            {
                list.start = start;
            }
        }
        (Name::LIST_ITEM_VALUE, Kind::Enter) => {}
        (_, Kind::Enter) => {
            let spread = compiler.loose.contains(&compiler.index);
            compiler.tail_push_after_prefix(Node::List(List {
                ordered: name == Name::LIST_ORDERED,
                start: None,
                spread: Some(spread),
                children: ArenaVec::new_in(compiler.alloc),
                position: None,
            }));
        }
        (_, Kind::Exit) => compiler.tail_pop_before_trailing_lines(false),
    }
}
