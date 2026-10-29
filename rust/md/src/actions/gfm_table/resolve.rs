use crate::models::event::{ContentType, Event, Kind, Link};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

// A cell as event indices: the divider before it, its start, its first and its last value
type Cell = (usize, usize, usize, usize);

/// Wrap tables, bodies and cells around the rows, and link each cell's text as text.
pub(crate) fn resolve_gfm_table(tokenizer: &mut Tokenizer) {
    let mut index = 0;
    let mut in_first_cell_awaiting_pipe = true;
    let mut in_row = false;
    let mut in_delimiter_row = false;
    let mut last_cell: Cell = (0, 0, 0, 0);
    let mut cell: Cell = (0, 0, 0, 0);
    let mut after_head_awaiting_first_body_row = false;
    let mut last_table_end = 0;
    let mut last_table_has_body = false;
    while index < tokenizer.events.len() {
        let event = &tokenizer.events[index];
        let name = event.name;
        let is_value = matches!(
            name,
            Name::DATA
                | Name::GFM_TABLE_DELIMITER_MARKER
                | Name::GFM_TABLE_DELIMITER_FILLER
        );
        if event.kind == Kind::Enter {
            if name == Name::GFM_TABLE_HEAD {
                after_head_awaiting_first_body_row = false;
                if last_table_end != 0 {
                    flush_table_end(tokenizer, last_table_end, last_table_has_body);
                    last_table_has_body = false;
                    last_table_end = 0;
                }
                let enter = create_event(tokenizer, index, Kind::Enter, Name::GFM_TABLE);
                tokenizer.map.add(index, 0, vec![enter]);
            } else if name == Name::GFM_TABLE_ROW || name == Name::GFM_TABLE_DELIMITER_ROW
            {
                in_delimiter_row = name == Name::GFM_TABLE_DELIMITER_ROW;
                in_row = true;
                in_first_cell_awaiting_pipe = true;
                last_cell = (0, 0, 0, 0);
                cell = (0, index + 1, 0, 0);
                if after_head_awaiting_first_body_row {
                    after_head_awaiting_first_body_row = false;
                    last_table_has_body = true;
                    let enter =
                        create_event(tokenizer, index, Kind::Enter, Name::GFM_TABLE_BODY);
                    tokenizer.map.add(index, 0, vec![enter]);
                }
            } else if in_row && is_value {
                in_first_cell_awaiting_pipe = false;
                if cell.2 == 0 {
                    if last_cell.1 != 0 {
                        cell.0 = cell.1;
                        flush_cell(tokenizer, last_cell, in_delimiter_row, None);
                        last_cell = (0, 0, 0, 0);
                    }
                    cell.2 = index;
                }
            } else if name == Name::GFM_TABLE_CELL_DIVIDER {
                if in_first_cell_awaiting_pipe {
                    in_first_cell_awaiting_pipe = false;
                } else {
                    if last_cell.1 != 0 {
                        cell.0 = cell.1;
                        flush_cell(tokenizer, last_cell, in_delimiter_row, None);
                    }
                    last_cell = cell;
                    cell = (last_cell.1, index, 0, 0);
                }
            }
        } else if name == Name::GFM_TABLE_HEAD {
            after_head_awaiting_first_body_row = true;
            last_table_end = index;
        } else if name == Name::GFM_TABLE_ROW || name == Name::GFM_TABLE_DELIMITER_ROW {
            in_row = false;
            last_table_end = index;
            if last_cell.1 != 0 {
                cell.0 = cell.1;
                flush_cell(tokenizer, last_cell, in_delimiter_row, Some(index));
            } else if cell.1 != 0 {
                flush_cell(tokenizer, cell, in_delimiter_row, Some(index));
            }
        } else if in_row && is_value {
            cell.3 = index;
        }
        index += 1;
    }
    if last_table_end != 0 {
        flush_table_end(tokenizer, last_table_end, last_table_has_body);
    }
    tokenizer
        .map
        .consume(&mut tokenizer.events, &tokenizer.parse_state.spare_events);
}

fn flush_cell(
    tokenizer: &mut Tokenizer,
    range: Cell,
    in_delimiter_row: bool,
    row_end: Option<usize>,
) {
    let (group_name, value_name) = if in_delimiter_row {
        (
            Name::GFM_TABLE_DELIMITER_CELL,
            Name::GFM_TABLE_DELIMITER_CELL_VALUE,
        )
    } else {
        (Name::GFM_TABLE_CELL, Name::GFM_TABLE_CELL_TEXT)
    };
    if range.0 != 0 {
        let exit = create_event(tokenizer, range.0, Kind::Exit, group_name);
        tokenizer.map.add(range.0, 0, vec![exit]);
    }
    let enter = create_event(tokenizer, range.1, Kind::Enter, group_name);
    tokenizer.map.add(range.1, 0, vec![enter]);
    if range.2 != 0 {
        let enter = create_event(tokenizer, range.2, Kind::Enter, value_name);
        tokenizer.map.add(range.2, 0, vec![enter]);
        if !in_delimiter_row {
            tokenizer.events[range.2].link = Some(Link {
                previous: None,
                next: None,
                content: ContentType::Text,
            });
            // One chunk from the first value's start to the last one's end
            if range.3 > range.2 + 1 {
                tokenizer
                    .map
                    .add(range.2 + 1, range.3 - range.2 - 1, vec![]);
            }
        }
        let exit = create_event(tokenizer, range.3, Kind::Exit, value_name);
        tokenizer.map.add(range.3 + 1, 0, vec![exit]);
    }
    if let Some(row_end) = row_end {
        let exit = create_event(tokenizer, row_end, Kind::Exit, group_name);
        tokenizer.map.add(row_end, 0, vec![exit]);
    }
}

fn flush_table_end(tokenizer: &mut Tokenizer, index: usize, body: bool) {
    let mut exits = vec![];
    if body {
        exits.push(create_event(
            tokenizer,
            index,
            Kind::Exit,
            Name::GFM_TABLE_BODY,
        ));
    }
    exits.push(create_event(tokenizer, index, Kind::Exit, Name::GFM_TABLE));
    tokenizer.map.add(index + 1, 0, exits);
}

fn create_event(tokenizer: &Tokenizer, at: usize, kind: Kind, name: Name) -> Event {
    Event {
        kind,
        name,
        point: tokenizer.events[at].point.clone(),
        link: None,
    }
}
