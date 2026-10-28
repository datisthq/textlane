use std::collections::HashMap;

use crate::helpers::character::{
    CharacterKind, classify_character, find_character_after, find_character_before,
};
use crate::models::event::{Event, Kind, Point};
use crate::models::token::Name;
use crate::utils::tokenizer::driver::Tokenizer;

/// One run of attention markers and whether it can open or close emphasis.
#[derive(Debug)]
struct Sequence {
    marker: u8,
    stack: usize,
    index: usize,
    start_point: Point,
    end_point: Point,
    size: usize,
    open: bool,
    close: bool,
}

// Openers that may pair with a closer: by enclosing tokens, marker, size modulo three (for
// the rule of three) and whether the opener can close too
type Bucket = (usize, u8, usize, bool);

/// Pair attention sequences into emphasis and strong, as CommonMark's delimiter run rules
/// say; sequences left over become data.
///
/// Each closer takes the nearest opener micromark would pick, but finds it at the top of a
/// stack of openers per bucket instead of scanning back, so input with many runs stays
/// linear.
pub(crate) fn resolve_attention(tokenizer: &mut Tokenizer) {
    let mut sequences = find_sequences(tokenizer);
    let mut buckets: HashMap<Bucket, Vec<usize>> = HashMap::new();
    // Every sequence that can still open, in order: a match closes the ones between
    let mut openers: Vec<usize> = vec![];
    for close in 0..sequences.len() {
        while sequences[close].close && sequences[close].size > 0 {
            let Some(open) = find_opener(&sequences, &mut buckets, close) else {
                break;
            };
            match_sequences(tokenizer, &mut sequences, open, close);
            while openers.last().is_some_and(|last| *last > open) {
                if let Some(between) = openers.pop() {
                    sequences[between].open = false;
                }
            }
            if sequences[open].size > 0 {
                let key = bucket_of(&sequences[open]);
                buckets.entry(key).or_default().push(open);
            }
        }
        if sequences[close].open && sequences[close].size > 0 {
            let key = bucket_of(&sequences[close]);
            buckets.entry(key).or_default().push(close);
            openers.push(close);
        }
    }
    for sequence in sequences.iter().filter(|sequence| sequence.size > 0) {
        tokenizer.events[sequence.index].name = Name::DATA;
        tokenizer.events[sequence.index + 1].name = Name::DATA;
    }
    tokenizer.map.consume(&mut tokenizer.events);
}

fn bucket_of(sequence: &Sequence) -> Bucket {
    (
        sequence.stack,
        sequence.marker,
        sequence.size % 3,
        sequence.close,
    )
}

// The nearest live opener in a bucket the closer may pair with
fn find_opener(
    sequences: &[Sequence],
    buckets: &mut HashMap<Bucket, Vec<usize>>,
    close: usize,
) -> Option<usize> {
    let closer = &sequences[close];
    let mut best: Option<usize> = None;
    for size in 0..3 {
        for can_close in [false, true] {
            // The rule of three: a run that both opens and closes pairs only when the
            // sizes do not add up to a multiple of three
            if (can_close || closer.open)
                && !closer.size.is_multiple_of(3)
                && (size + closer.size).is_multiple_of(3)
            {
                continue;
            }
            let key = (closer.stack, closer.marker, size, can_close);
            let Some(bucket) = buckets.get_mut(&key) else {
                continue;
            };
            // Drop openers that closed, were used up or moved to another bucket
            while let Some(top) = bucket.last() {
                let sequence = &sequences[*top];
                if sequence.open && sequence.size > 0 && bucket_of(sequence) == key {
                    break;
                }
                bucket.pop();
            }
            if let Some(top) = bucket.last()
                && best.is_none_or(|best| *top > best)
            {
                best = Some(*top);
            }
        }
    }
    best
}

fn find_sequences(tokenizer: &Tokenizer) -> Vec<Sequence> {
    let bytes = tokenizer.parse_state.bytes;
    let mut stack = vec![];
    let mut stacks: HashMap<Vec<usize>, usize> = HashMap::new();
    let mut sequences = vec![];
    for (index, enter) in tokenizer.events.iter().enumerate() {
        if enter.name == Name::ATTENTION_SEQUENCE {
            if enter.kind == Kind::Enter {
                let exit = &tokenizer.events[index + 1];
                let marker = bytes[enter.point.index];
                let before_char = find_character_before(bytes, enter.point.index);
                let before = classify_character(before_char);
                let after_char = find_character_after(bytes, exit.point.index);
                let after = classify_character(after_char);
                // micromark lets extension markers (GFM's `~`) open and close next to
                // punctuation too, but never `*` or `_`
                let open = after == CharacterKind::Other
                    || (after == CharacterKind::Punctuation
                        && before != CharacterKind::Other);
                let close = before == CharacterKind::Other
                    || (before == CharacterKind::Punctuation
                        && after != CharacterKind::Other);
                // `_` may not open or close inside a word
                let (open, close) = if marker == b'_' {
                    (
                        open && (before != CharacterKind::Other || !close),
                        close && (after != CharacterKind::Other || !open),
                    )
                } else {
                    (open, close)
                };
                let next_id = stacks.len();
                let stack_id = *stacks.entry(stack.clone()).or_insert(next_id);
                sequences.push(Sequence {
                    index,
                    stack: stack_id,
                    start_point: enter.point.clone(),
                    end_point: exit.point.clone(),
                    size: exit.point.index - enter.point.index,
                    open,
                    close,
                    marker,
                });
            }
        } else if enter.kind == Kind::Enter {
            stack.push(index);
        } else {
            stack.pop();
        }
    }
    sequences
}

fn match_sequences(
    tokenizer: &mut Tokenizer,
    sequences: &mut [Sequence],
    open: usize,
    close: usize,
) {
    let take = if sequences[open].size > 1 && sequences[close].size > 1 {
        2
    } else {
        1
    };
    let (group_name, sequence_name, text_name) = if take == 1 {
        (Name::EMPHASIS, Name::EMPHASIS_SEQUENCE, Name::EMPHASIS_TEXT)
    } else {
        (Name::STRONG, Name::STRONG_SEQUENCE, Name::STRONG_TEXT)
    };
    let open_index = sequences[open].index;
    let close_index = sequences[close].index;
    let open_exit = sequences[open].end_point.clone();
    let close_enter = sequences[close].start_point.clone();
    sequences[open].size -= take;
    sequences[close].size -= take;
    sequences[open].end_point.column -= take;
    sequences[open].end_point.index -= take;
    sequences[close].start_point.column += take;
    sequences[close].start_point.index += take;

    let event = |kind, name, point: &Point| Event {
        kind,
        name,
        point: point.clone(),
        link: None,
    };
    let open_point = sequences[open].end_point.clone();
    tokenizer.map.add_before(
        open_index + 2,
        0,
        vec![
            event(Kind::Enter, group_name, &open_point),
            event(Kind::Enter, sequence_name, &open_point),
            event(Kind::Exit, sequence_name, &open_exit),
            event(Kind::Enter, text_name, &open_exit),
        ],
    );
    let close_point = sequences[close].start_point.clone();
    tokenizer.map.add(
        close_index,
        0,
        vec![
            event(Kind::Exit, text_name, &close_enter),
            event(Kind::Enter, sequence_name, &close_enter),
            event(Kind::Exit, sequence_name, &close_point),
            event(Kind::Exit, group_name, &close_point),
        ],
    );

    // A used-up sequence's own events go; a partly used one shrinks
    if sequences[close].size == 0 {
        tokenizer.map.add(close_index, 2, vec![]);
    } else {
        tokenizer.events[close_index].point = close_point;
    }
    if sequences[open].size == 0 {
        tokenizer.map.add(open_index, 2, vec![]);
    } else {
        tokenizer.events[open_index + 1].point = sequences[open].end_point.clone();
    }
}
