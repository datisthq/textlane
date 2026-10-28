use textlane_core::mdast::Node;
use textlane_core::{Point, Position};

use crate::models::event::Kind;
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// A task list check sets whether its list item is checked; the item's paragraph is open
/// right inside it.
pub(crate) fn compile_gfm_task_list_item_check(
    compiler: &mut Compiler,
    name: Name,
    kind: Kind,
) {
    if kind != Kind::Exit {
        return;
    }
    if let Some(Node::ListItem(item)) = compiler.penultimate_mut() {
        item.checked = Some(name == Name::TASK_LIST_CHECK_VALUE_CHECKED);
        compiler.task_list_check_seen = true;
    }
}

/// Drop the whitespace after a task list check from the paragraph that just closed into
/// its list item, as mdast-util-gfm-task-list-item does.
pub(crate) fn strip_task_list_item_space(compiler: &mut Compiler) {
    if !std::mem::take(&mut compiler.task_list_check_seen) {
        return;
    }
    let source = compiler.source.as_bytes();
    let Some(Node::Paragraph(paragraph)) = compiler
        .tail_mut()
        .children_mut()
        .and_then(|children| children.last_mut())
    else {
        return;
    };
    let Some(Node::Text(head)) = paragraph.children.first_mut() else {
        return;
    };
    let first = head.value.chars().next().map_or(0, char::len_utf8);
    head.value = &head.value[first..];
    if head.value.is_empty() {
        paragraph.children.remove(0);
        return;
    }
    // Upstream moves the start one column and offset on, even past a tab or line ending:
    // a point into a character is one column and offset past its start
    if let Some(Position::Span { start, .. }) = &mut head.position {
        let index = start.offset();
        *start = if source.get(index) == Some(&b' ') {
            Point::new(index + 1, 0)
        } else {
            Point::new(index, start.virtual_spaces() + 1)
        };
        if let Some(Position::Span {
            start: paragraph_start,
            ..
        }) = &mut paragraph.position
        {
            *paragraph_start = *start;
        }
    }
}
