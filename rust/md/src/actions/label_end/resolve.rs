use crate::models::event::{Event, Kind};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::{Label, LabelKind, LabelStart, Tokenizer};

/// Wrap each closed label in a link or image; label starts that never closed become data.
pub(crate) fn resolve_label(tokenizer: &mut Tokenizer) {
    let labels = tokenizer.tokenize_state.labels.split_off(0);
    inject_labels(tokenizer, &labels);
    let starts = tokenizer.tokenize_state.label_starts.split_off(0);
    mark_as_data(tokenizer, &starts);
    let starts = tokenizer.tokenize_state.label_starts_loose.split_off(0);
    mark_as_data(tokenizer, &starts);
    tokenizer.tokenize_state.label_starts_inactive_before = 0;
    tokenizer.map.consume(&mut tokenizer.events);
}

fn inject_labels(tokenizer: &mut Tokenizer, labels: &[Label]) {
    let event = |kind, name, index: usize, events: &[Event]| Event {
        kind,
        name,
        point: events[index].point,
        link: None,
    };
    for label in labels {
        let group = match label.kind {
            LabelKind::Image => Name::IMAGE,
            LabelKind::Link => Name::LINK,
        };
        let events = &tokenizer.events;
        let open = vec![
            event(Kind::Enter, group, label.start.0, events),
            event(Kind::Enter, Name::LABEL, label.start.0, events),
        ];
        let text = (label.start.1 != label.end.0).then(|| {
            (
                event(Kind::Enter, Name::LABEL_TEXT, label.start.1, events),
                event(Kind::Exit, Name::LABEL_TEXT, label.end.0, events),
            )
        });
        let label_exit = event(Kind::Exit, Name::LABEL, label.end.0 + 3, events);
        let group_exit = event(Kind::Exit, group, label.end.1, events);
        tokenizer.map.add(label.start.0, 0, open);
        if let Some((text_enter, text_exit)) = text {
            tokenizer
                .map
                .add_before(label.start.1 + 1, 0, vec![text_enter]);
            tokenizer.map.add(label.end.0, 0, vec![text_exit]);
        }
        tokenizer.map.add(label.end.0 + 4, 0, vec![label_exit]);
        tokenizer.map.add(label.end.1 + 1, 0, vec![group_exit]);
    }
}

fn mark_as_data(tokenizer: &mut Tokenizer, starts: &[LabelStart]) {
    for start in starts {
        let (enter, exit) = start.start;
        let data = vec![
            Event {
                kind: Kind::Enter,
                name: Name::DATA,
                point: tokenizer.events[enter].point,
                link: None,
            },
            Event {
                kind: Kind::Exit,
                name: Name::DATA,
                point: tokenizer.events[exit].point,
                link: None,
            },
        ];
        tokenizer.map.add(enter, exit - enter + 1, data);
    }
}
