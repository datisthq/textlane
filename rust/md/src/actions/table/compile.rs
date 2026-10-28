use textlane_core::ArenaVec;
use textlane_core::mdast::{AlignKind, Node, Table, TableCell, TableRow};

use crate::models::event::{Event, Kind};
use crate::models::token::Name;
use crate::utils::compiler::stack::Compiler;

/// Open and close a table, its rows and their cells.
pub(crate) fn compile_table(compiler: &mut Compiler, name: Name, kind: Kind) {
    if kind == Kind::Exit {
        compiler.tail_pop();
        if name == Name::GFM_TABLE {
            compiler.in_table = false;
        }
        return;
    }
    let children = ArenaVec::new_in(compiler.alloc);
    let node = match name {
        Name::GFM_TABLE => {
            compiler.in_table = true;
            let mut align = ArenaVec::new_in(compiler.alloc);
            align.extend(read_align(compiler.events, compiler.index));
            Node::Table(Table {
                align,
                children,
                position: None,
            })
        }
        Name::GFM_TABLE_ROW => Node::TableRow(TableRow {
            children,
            position: None,
        }),
        _ => Node::TableCell(TableCell {
            children,
            position: None,
        }),
    };
    compiler.tail_push(node);
}

// Each column's alignment, from the markers in the delimiter row after `index`
fn read_align(events: &[Event], mut index: usize) -> Vec<AlignKind> {
    let mut in_delimiter_row = false;
    let mut align = vec![];
    while index < events.len() {
        let event = &events[index];
        if !in_delimiter_row {
            in_delimiter_row =
                event.kind == Kind::Enter && event.name == Name::GFM_TABLE_DELIMITER_ROW;
        } else if event.kind == Kind::Enter {
            if event.name == Name::GFM_TABLE_DELIMITER_CELL_VALUE {
                align.push(
                    if events[index + 1].name == Name::GFM_TABLE_DELIMITER_MARKER {
                        AlignKind::Left
                    } else {
                        AlignKind::None
                    },
                );
            }
        } else if event.name == Name::GFM_TABLE_DELIMITER_CELL_VALUE {
            if events[index - 1].name == Name::GFM_TABLE_DELIMITER_MARKER
                && let Some(last) = align.last_mut()
            {
                *last = if *last == AlignKind::Left {
                    AlignKind::Center
                } else {
                    AlignKind::Right
                };
            }
        } else if event.name == Name::GFM_TABLE_DELIMITER_ROW {
            break;
        }
        index += 1;
    }
    align
}
