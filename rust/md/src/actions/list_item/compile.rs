use textlane_core::ArenaVec;
use textlane_core::mdast::{ListItem, Node};

use crate::models::event::Kind;
use crate::utils::compiler::stack::Compiler;

/// Open and close a list item.
pub(crate) fn compile_list_item(compiler: &mut Compiler, kind: Kind) {
    match kind {
        Kind::Enter => {
            let spread = compiler.loose.contains(&compiler.index);
            compiler.tail_push_after_prefix(Node::ListItem(ListItem {
                spread: Some(spread),
                checked: None,
                children: ArenaVec::new_in(compiler.alloc),
                position: None,
            }));
        }
        Kind::Exit => compiler.tail_pop_before_trailing_lines(true),
    }
}
