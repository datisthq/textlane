use textlane_core::hast::{Element, Node};
use textlane_core::mdast::AlignKind;

use crate::helpers::text::read_text;
use crate::helpers::value::read_value;
use crate::utils::converter::state::Converter;
use crate::utils::converter::store::{Converted, Id, Kind};

// Spans and ragged rows add empty cells by the area they cover, not the input they take;
// past this many in one table, the rest of its rows stay as they are
const MAX_ADDED_CELLS: usize = 1 << 19;

/// `<table>`: a table, padded to a rectangle around cells that span several; a table in a
/// table is its text.
pub(crate) fn convert_table<'a, 'n>(
    state: &mut Converter<'a, 'n, '_>,
    element: &'n Element<'a>,
) -> Converted {
    if state.in_table {
        let value = state.alloc.alloc_str(&read_text(element));
        return Converted::One(state.store.add_text(value, element.position));
    }
    state.in_table = true;
    let (align, headless) = inspect(element);
    let children = state.all(&element.children);
    let mut rows = state.wrap_specific_content(children, Kind::TableRow);
    if headless {
        rows.insert(0, state.store.add(Kind::TableRow, vec![], None));
    }
    for row in &rows {
        let cells = state.store.take_children(*row);
        let cells = state.wrap_specific_content(cells, Kind::TableCell(None));
        state.store.set_children(*row, cells);
    }
    let columns = pad_spans(state, &rows);
    let mut align: Vec<AlignKind> = align
        .into_iter()
        .map(|align| align.unwrap_or(AlignKind::None))
        .collect();
    if align.len() < columns {
        align.resize(columns, AlignKind::None);
    }
    state.in_table = false;
    Converted::One(state.store.add(Kind::Table(align), rows, element.position))
}

// Upstream splices an empty cell into each row and column a cell spans, then pads every
// row to the widest. Each row's splices from the rows above are replayed in one pass and
// its own spans added as it is rebuilt, so nothing is spliced one cell at a time.
fn pad_spans(state: &mut Converter<'_, '_, '_>, rows: &[Id]) -> usize {
    let mut budget = MAX_ADDED_CELLS;
    let mut pending: Vec<Vec<(usize, usize)>> = vec![vec![]; rows.len()];
    let mut columns = 1;
    for (row_index, row) in rows.iter().enumerate() {
        let cells = state.store.take_children(*row);
        let inserts = std::mem::take(&mut pending[row_index]);
        let cells = replay_inserts(state, cells, &inserts);
        let mut built = Vec::with_capacity(cells.len());
        for cell in cells {
            let column = built.len();
            built.push(cell);
            let Kind::TableCell(Some((row_span, column_span))) =
                state.store.items[cell].kind
            else {
                continue;
            };
            if row_span <= 1 && column_span <= 1 {
                continue;
            }
            let own = (column_span - 1).min(budget);
            budget -= own;
            for _ in 0..own {
                built.push(state.store.add(Kind::TableCell(None), vec![], None));
            }
            let row_end = row_index.saturating_add(row_span);
            for other in pending.iter_mut().take(row_end).skip(row_index + 1) {
                let count = column_span.min(budget);
                if count == 0 {
                    break;
                }
                budget -= count;
                other.push((column, count));
            }
        }
        columns = columns.max(built.len());
        state.store.set_children(*row, built);
    }
    for row in rows {
        let missing = columns.saturating_sub(state.store.items[*row].children.len());
        let missing = missing.min(budget);
        budget -= missing;
        for _ in 0..missing {
            let empty = state.store.add(Kind::TableCell(None), vec![], None);
            state.store.items[*row].children.push(empty);
        }
    }
    columns
}

// Apply `count` empty cells spliced in at `position` (clamped to the row's length then),
// in order, by finding from the last splice back which free slots each one took
fn replay_inserts(
    state: &mut Converter<'_, '_, '_>,
    cells: Vec<Id>,
    inserts: &[(usize, usize)],
) -> Vec<Id> {
    if inserts.is_empty() {
        return cells;
    }
    let mut length = cells.len();
    let mut clamped = Vec::with_capacity(inserts.len());
    for (position, count) in inserts {
        clamped.push(((*position).min(length), *count));
        length += count;
    }
    let mut free = FreeSlots::new(length);
    let mut empty = vec![false; length];
    for (position, count) in clamped.iter().rev() {
        for _ in 0..*count {
            let slot = free.take(*position);
            empty[slot] = true;
        }
    }
    let mut cells = cells.into_iter();
    let mut built = Vec::with_capacity(length);
    for is_empty in empty {
        let cell = match is_empty {
            true => state.store.add(Kind::TableCell(None), vec![], None),
            false => cells.next().unwrap_or_default(),
        };
        built.push(cell);
    }
    built
}

// A Fenwick tree over slots, each free until taken
struct FreeSlots {
    tree: Vec<usize>,
}

impl FreeSlots {
    fn new(length: usize) -> Self {
        let mut tree = vec![0; length + 1];
        for index in 1..=length {
            tree[index] += 1;
            let parent = index + (index & index.wrapping_neg());
            if parent <= length {
                tree[parent] += tree[index];
            }
        }
        Self { tree }
    }

    // Take the free slot with `rank` free slots before it
    fn take(&mut self, rank: usize) -> usize {
        let length = self.tree.len() - 1;
        let mut position = 0;
        let mut remaining = rank + 1;
        let mut step = length.next_power_of_two();
        while step > 0 {
            let next = position + step;
            if next <= length && self.tree[next] < remaining {
                position = next;
                remaining -= self.tree[next];
            }
            step /= 2;
        }
        let mut index = position + 1;
        while index <= length {
            self.tree[index] -= 1;
            index += index & index.wrapping_neg();
        }
        position
    }
}

// The alignment of each column, from the first cell in it that sets one, and whether the
// table has no head: no `<thead>` and no `<th>` in its first two rows
fn inspect(table: &Element<'_>) -> (Vec<Option<AlignKind>>, bool) {
    let mut align: Vec<Option<AlignKind>> = vec![None];
    let mut headless = true;
    let mut row_index = 0;
    let mut cell_index = 0;
    let mut stack: Vec<&Node<'_>> = table.children.iter().rev().collect();
    while let Some(node) = stack.pop() {
        let Node::Element(element) = node else {
            continue;
        };
        match element.tag_name {
            "table" => continue,
            "th" | "td" => {
                if align.get(cell_index).copied().flatten().is_none() {
                    let value = match &*read_value(element, "align") {
                        "" => Some(AlignKind::None),
                        "center" => Some(AlignKind::Center),
                        "left" => Some(AlignKind::Left),
                        "right" => Some(AlignKind::Right),
                        _ => None,
                    };
                    if let Some(value) = value {
                        if align.len() <= cell_index {
                            align.resize(cell_index + 1, None);
                        }
                        align[cell_index] =
                            Some(value).filter(|value| *value != AlignKind::None);
                    }
                }
                if headless && row_index < 2 && element.tag_name == "th" {
                    headless = false;
                }
                cell_index += 1;
            }
            "thead" => headless = false,
            "tr" => {
                row_index += 1;
                cell_index = 0;
            }
            _ => {}
        }
        stack.extend(element.children.iter().rev());
    }
    (align, headless)
}
