use std::collections::HashMap;

use crate::helpers::character::{
    CharacterKind, classify_character_unit, find_character_after, find_character_before,
};
use crate::models::event::{Event, Kind, Point};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// A run of one or two `~`, what it sits in and whether it can open or close.
#[derive(Debug)]
struct Sequence {
    index: usize,
    parent: usize,
    size: usize,
    open: bool,
    close: bool,
}

/// Pair `~` sequences of the same size into strikethrough; sequences left over become data.
///
/// micromark walks back from each closer to the nearest opener of its size and turns the
/// sequences between into data; a stack of openers per size and parent finds the same one
/// without walking.
pub(crate) fn resolve_gfm_strikethrough(tokenizer: &mut Tokenizer) {
    let sequences = find_sequences(tokenizer);
    let mut by_size: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    let mut by_parent: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut live = vec![false; sequences.len()];
    let mut matched = vec![false; sequences.len()];
    for close in 0..sequences.len() {
        let sequence = &sequences[close];
        let key = (sequence.parent, sequence.size);
        if sequence.close
            && let Some(open) = pop_opener(by_size.get_mut(&key), &live)
        {
            if let Some(openers) = by_parent.get_mut(&sequence.parent) {
                while let Some(top) = openers.pop_if(|top| *top >= open) {
                    live[top] = false;
                }
            }
            matched[open] = true;
            matched[close] = true;
            wrap(tokenizer, &sequences[open], sequence);
            continue;
        }
        if sequence.open {
            live[close] = true;
            by_size.entry(key).or_default().push(close);
            by_parent.entry(sequence.parent).or_default().push(close);
        }
    }
    for (sequence, matched) in sequences.iter().zip(matched) {
        let name = if matched {
            Name::STRIKETHROUGH_SEQUENCE
        } else {
            Name::DATA
        };
        tokenizer.events[sequence.index].name = name;
        tokenizer.events[sequence.index + 1].name = name;
    }
    tokenizer.map.consume(&mut tokenizer.events);
}

// The nearest opener still live, dropping those that closed or turned into data
fn pop_opener(openers: Option<&mut Vec<usize>>, live: &[bool]) -> Option<usize> {
    let openers = openers?;
    while let Some(top) = openers.pop() {
        if live[top] {
            return Some(top);
        }
    }
    None
}

fn find_sequences(tokenizer: &Tokenizer) -> Vec<Sequence> {
    let bytes = tokenizer.parse_state.bytes;
    let mut parents = vec![usize::MAX];
    let mut sequences = vec![];
    for (index, event) in tokenizer.events.iter().enumerate() {
        if event.name == Name::STRIKETHROUGH_SEQUENCE_TEMPORARY {
            if event.kind == Kind::Enter {
                let exit = &tokenizer.events[index + 1];
                let before = classify_character_unit(find_character_before(
                    bytes,
                    event.point.index,
                ));
                let after = classify_character_unit(find_character_after(
                    bytes,
                    exit.point.index,
                ));
                sequences.push(Sequence {
                    index,
                    parent: parents.last().copied().unwrap_or(usize::MAX),
                    size: exit.point.index - event.point.index,
                    open: after == CharacterKind::Other
                        || (after == CharacterKind::Punctuation
                            && before != CharacterKind::Other),
                    close: before == CharacterKind::Other
                        || (before == CharacterKind::Punctuation
                            && after != CharacterKind::Other),
                });
            }
        } else if event.kind == Kind::Enter {
            parents.push(index);
        } else {
            parents.pop();
        }
    }
    sequences
}

fn wrap(tokenizer: &mut Tokenizer, open: &Sequence, close: &Sequence) {
    let event = |kind, name, point: &Point| Event {
        kind,
        name,
        point: point.clone(),
        link: None,
    };
    let events = &tokenizer.events;
    let open_start = events[open.index].point.clone();
    let open_end = events[open.index + 1].point.clone();
    let close_start = events[close.index].point.clone();
    let close_end = events[close.index + 1].point.clone();
    let map = &mut tokenizer.map;
    map.add(
        open.index,
        0,
        vec![event(Kind::Enter, Name::STRIKETHROUGH, &open_start)],
    );
    map.add_before(
        open.index + 2,
        0,
        vec![event(Kind::Enter, Name::STRIKETHROUGH_TEXT, &open_end)],
    );
    map.add(
        close.index,
        0,
        vec![event(Kind::Exit, Name::STRIKETHROUGH_TEXT, &close_start)],
    );
    map.add(
        close.index + 2,
        0,
        vec![event(Kind::Exit, Name::STRIKETHROUGH, &close_end)],
    );
}
