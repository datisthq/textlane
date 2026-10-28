use crate::models::event::{Event, Kind};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// Merge the data between a heading's sequences into one heading text: closing `#`s that
/// turned out to be text join it.
pub(crate) fn resolve_heading_atx(tokenizer: &mut Tokenizer) {
    let mut heading_inside = false;
    let mut data_start: Option<usize> = None;
    let mut data_end: Option<usize> = None;
    for index in 0..tokenizer.events.len() {
        let event = &tokenizer.events[index];
        if event.name == Name::HEADING_ATX {
            if event.kind == Kind::Enter {
                heading_inside = true;
            } else {
                if let (Some(start), Some(end)) = (data_start, data_end) {
                    let enter = Event {
                        kind: Kind::Enter,
                        name: Name::HEADING_ATX_TEXT,
                        point: tokenizer.events[start].point.clone(),
                        link: None,
                    };
                    let exit = Event {
                        kind: Kind::Exit,
                        name: Name::HEADING_ATX_TEXT,
                        point: tokenizer.events[end].point.clone(),
                        link: None,
                    };
                    tokenizer.map.add(start, 0, vec![enter]);
                    tokenizer.map.add(start + 1, end - start - 1, vec![]);
                    tokenizer.map.add(end + 1, 0, vec![exit]);
                }
                heading_inside = false;
                data_start = None;
                data_end = None;
            }
        } else if heading_inside && event.name == Name::DATA {
            if event.kind == Kind::Enter {
                if data_start.is_none() {
                    data_start = Some(index);
                }
            } else {
                data_end = Some(index);
            }
        }
    }
    tokenizer.map.consume(&mut tokenizer.events);
}
