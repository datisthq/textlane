use crate::models::construct::State;
use crate::models::event::{ContentType, Event, Kind, Link, Point};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// `]` after `![^a` that no image took: the `!` becomes text and `[^a]` a footnote call,
/// as micromark-extension-gfm-footnote's `gfmPotentialFootnoteCall` does.
pub(crate) fn tokenize_gfm_potential_footnote_call(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.gfm.is_none() || tokenizer.current != Some(b']') {
        return State::Nok;
    }
    let Some((enter, exit)) = tokenizer.tokenize_state.gfm_footnote_image else {
        return State::Nok;
    };
    let parse_state = tokenizer.parse_state;
    let bytes = parse_state.bytes;
    let from = tokenizer.events[exit].point.index;
    let to = tokenizer.point.index;
    // micromark also takes `![ ^a]`, its marker then being the space, or a line ending
    // that would need a position on the next line; only `^` right after `[` is taken here
    if bytes.get(from as usize) != Some(&b'^')
        || from + 1 >= to
        || matches!(bytes[(from + 1) as usize], b'\t' | b'\n' | b'\r' | b' ')
        || !parse_state.is_gfm_footnote_defined(from as usize + 1, to as usize)
    {
        return State::Nok;
    }
    tokenizer.enter(Name::GFM_FOOTNOTE_CALL_LABEL_MARKER);
    tokenizer.consume();
    tokenizer.exit(Name::GFM_FOOTNOTE_CALL_LABEL_MARKER);
    rewrite_image_start(tokenizer, enter);
    let state = &mut tokenizer.tokenize_state;
    state.gfm_footnote_image = None;
    if let Some(index) = state
        .label_starts_loose
        .iter()
        .rposition(|start| start.start == (enter, exit))
    {
        state.label_starts_loose.remove(index);
    }
    State::Ok
}

// Nothing after the image start is a label start or a label, so its events can be
// replaced here: `!` as data, then a call around a string up to the `]` just consumed
fn rewrite_image_start(tokenizer: &mut Tokenizer, enter: usize) {
    let events = &tokenizer.events;
    let len = events.len();
    let point = |index: usize| events[index].point;
    let bracket_end = point(enter + 4);
    let marker_end = Point {
        line: bracket_end.line,
        column: bracket_end.column + 1,
        index: bracket_end.index + 1,
        vs: 0,
    };
    let string_end = point(len - 2);
    let event = |kind, name, point| Event {
        kind,
        name,
        point,
        link: None,
    };
    let mut replacement = vec![
        event(Kind::Enter, Name::DATA, point(enter + 1)),
        event(Kind::Exit, Name::DATA, point(enter + 2)),
        event(Kind::Enter, Name::GFM_FOOTNOTE_CALL, point(enter + 3)),
        event(
            Kind::Enter,
            Name::GFM_FOOTNOTE_CALL_LABEL_MARKER,
            point(enter + 3),
        ),
        event(
            Kind::Exit,
            Name::GFM_FOOTNOTE_CALL_LABEL_MARKER,
            bracket_end,
        ),
        event(Kind::Enter, Name::GFM_FOOTNOTE_CALL_MARKER, bracket_end),
        event(Kind::Exit, Name::GFM_FOOTNOTE_CALL_MARKER, marker_end),
        event(Kind::Enter, Name::GFM_FOOTNOTE_CALL_STRING, marker_end),
        Event {
            kind: Kind::Enter,
            name: Name::DATA,
            point: marker_end,
            link: Some(Link {
                previous: None,
                next: None,
                content: ContentType::String,
            }),
        },
        event(Kind::Exit, Name::DATA, string_end),
        event(Kind::Exit, Name::GFM_FOOTNOTE_CALL_STRING, string_end),
        events[len - 2],
        events[len - 1],
        event(Kind::Exit, Name::GFM_FOOTNOTE_CALL, point(len - 1)),
    ];
    tokenizer.events.truncate(enter);
    tokenizer.events.append(&mut replacement);
}
