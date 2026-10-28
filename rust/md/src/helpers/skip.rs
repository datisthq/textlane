use crate::models::event::{Event, Kind};
use crate::models::token::Name;

/// The index of the first event at or after `index` named one of `names`.
pub(crate) fn skip_to(events: &[Event], mut index: usize, names: &[Name]) -> usize {
    while index < events.len() && !names.contains(&events[index].name) {
        index += 1;
    }
    index
}

/// Walk back from `index` over whole tokens named one of `names`.
pub(crate) fn skip_opt_back(events: &[Event], mut index: usize, names: &[Name]) -> usize {
    loop {
        let event = &events[index];
        if !names.contains(&event.name) || event.kind != Kind::Exit {
            return index;
        }
        let name = event.name;
        let mut balance = 0usize;
        loop {
            balance = if events[index].kind == Kind::Exit {
                balance + 1
            } else {
                balance - 1
            };
            let at_start = events[index].name == name && balance == 0;
            if index == 0 {
                return 0;
            }
            index -= 1;
            if at_start {
                break;
            }
        }
    }
}
