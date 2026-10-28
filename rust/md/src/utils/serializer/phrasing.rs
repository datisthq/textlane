use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::helpers::character::CharacterKind;
use crate::utils::serializer::state::{classify_unit, encode_character_reference};

/// A rendered phrasing child, or an emphasis or strong, by its index in the attention
/// store, whose markers are chosen once its whole phrasing is known.
#[derive(Debug)]
pub(crate) enum Item {
    Text(String),
    Attention(usize),
}

/// Kept flat in one store: nested items would drop recursively.
#[derive(Debug)]
pub(crate) struct Attention {
    pub(crate) sequences: Vec<String>,
    pub(crate) children: Vec<Item>,
}

#[derive(Clone, Debug)]
struct Token<'i> {
    value: Cow<'i, str>,
    attention: Option<usize>,
}

struct Run {
    tokens: Vec<usize>,
    markers: Vec<usize>,
    start: usize,
    end: usize,
    open: bool,
    close: bool,
    split: bool,
}

struct Mistake {
    attention: Vec<usize>,
    index: usize,
}

type Chosen = HashMap<usize, usize>;

// Each trial renders and checks the whole phrasing again, so adjacent attention needing a
// fix each is quadratic upstream; past this much work per token the fixes found so far stand
const TRIAL_WORK_PER_TOKEN: usize = 4;
const TRIAL_WORK_BASE: usize = 4096;

/// Join `items`, picking for each attention the first of its sequences that parses back as
/// that attention, as mdast-util-to-markdown's `containerPhrasing` does.
pub(crate) fn serialize_items(
    items: &[Item],
    store: &[Attention],
    before: &str,
    after: &str,
) -> String {
    let mut chosen = Chosen::new();
    let initial = render_tokens(items, store, &chosen, before, after);
    let mut budget = initial.len() * TRIAL_WORK_PER_TOKEN + TRIAL_WORK_BASE;
    let mut tokens = initial.clone();
    let mut mistake = check(&tokens, before, after);
    while let Some(current) = mistake {
        match improve(items, store, &chosen, &current, before, after, &mut budget) {
            Improvement::Found(trial, next_tokens, next) => {
                chosen = trial;
                tokens = next_tokens;
                mistake = next;
            }
            Improvement::None => {
                tokens = initial;
                break;
            }
            Improvement::OutOfBudget => break,
        }
    }
    tokens
        .into_iter()
        .map(|token| token.value)
        .collect::<String>()
}

enum Improvement<'i> {
    Found(Chosen, Vec<Token<'i>>, Option<Mistake>),
    None,
    OutOfBudget,
}

fn read_sequence<'i>(store: &'i [Attention], id: usize, chosen: &Chosen) -> &'i str {
    let index = chosen.get(&id).copied().unwrap_or(0);
    store
        .get(id)
        .and_then(|attention| attention.sequences.get(index))
        .map_or("", String::as_str)
}

fn render_tokens<'i>(
    items: &'i [Item],
    store: &'i [Attention],
    chosen: &Chosen,
    before: &str,
    after: &str,
) -> Vec<Token<'i>> {
    let mut tokens = Vec::new();
    render(items, store, chosen, before, after, &mut tokens);
    tokens
}

// Nested attention renders into the same `tokens`: moving each level's tokens into its
// parent's is quadratic in the nesting depth
fn render<'i>(
    items: &'i [Item],
    store: &'i [Attention],
    chosen: &Chosen,
    before: &str,
    after: &str,
    tokens: &mut Vec<Token<'i>>,
) {
    let level = tokens.len();
    let mut encode_after: Option<char> = None;
    let mut previous: Option<char> = before.chars().next_back();
    for (index, item) in items.iter().enumerate() {
        let id = match item {
            Item::Text(value) => {
                let mut token = Token {
                    value: Cow::Borrowed(value),
                    attention: None,
                };
                if encode_after.is_some() && encode_after == value.chars().next() {
                    encode_token(&mut token, true);
                }
                encode_after = None;
                if let Some(last) = token.value.chars().next_back() {
                    previous = Some(last);
                }
                tokens.push(token);
                continue;
            }
            Item::Attention(id) => *id,
        };
        let sequence = read_sequence(store, id, chosen);
        let marker = sequence.chars().next();
        let outside_before = previous;
        let outside_after = match items.get(index + 1) {
            None => after.chars().next(),
            Some(Item::Text(value)) => value.chars().next(),
            Some(Item::Attention(next)) => {
                read_sequence(store, *next, chosen).chars().next()
            }
        };
        let opening = tokens.len();
        tokens.push(Token {
            value: Cow::Borrowed(sequence),
            attention: Some(id),
        });
        let inside = tokens.len();
        if let Some(attention) = store.get(id) {
            stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
                render(
                    &attention.children,
                    store,
                    chosen,
                    sequence,
                    sequence,
                    tokens,
                );
            });
        }
        let head = tokens
            .get(inside)
            .and_then(|token| token.value.chars().next());
        let open = read_encode_info(outside_before, head, marker);
        if open.inside {
            encode_edge(&mut tokens[inside..], true);
        }
        let tail = tokens[inside..]
            .last()
            .and_then(|token| token.value.chars().next_back());
        let close = read_encode_info(outside_after, tail, marker);
        if close.inside {
            encode_edge(&mut tokens[inside..], false);
        }
        if open.outside && !matches!(outside_before, Some('\n' | '\r')) {
            encode_edge(&mut tokens[level..opening], false);
        }
        if close.outside {
            encode_after = outside_after;
        }
        tokens.push(Token {
            value: Cow::Borrowed(sequence),
            attention: Some(id),
        });
        previous = sequence.chars().next_back();
    }
}

fn encode_edge(tokens: &mut [Token], start: bool) {
    let token = if start {
        tokens.first_mut()
    } else {
        tokens.last_mut()
    };
    if let Some(token) = token
        && token.attention.is_none()
    {
        encode_token(token, start);
    }
}

fn encode_token(token: &mut Token, start: bool) {
    if start {
        let Some(character) = token.value.chars().next() else {
            return;
        };
        let rest = &token.value[character.len_utf8()..];
        token.value =
            Cow::Owned(format!("{}{rest}", encode_character_reference(character)));
    } else {
        let Some(character) = token.value.chars().next_back() else {
            return;
        };
        let mut value =
            token.value[..token.value.len() - character.len_utf8()].to_string();
        let escapes = value
            .bytes()
            .rev()
            .take_while(|byte| *byte == b'\\')
            .count();
        // An odd run of backslashes would escape the reference's `&`
        if escapes % 2 == 1 {
            value.push('\\');
        }
        value.push_str(&encode_character_reference(character));
        token.value = Cow::Owned(value);
    }
}

struct EncodeInfo {
    inside: bool,
    outside: bool,
}

// Whether the characters inside and outside a marker must become references for the
// marker to open or close
fn read_encode_info(
    outside: Option<char>,
    inside: Option<char>,
    marker: Option<char>,
) -> EncodeInfo {
    let both = |value| EncodeInfo {
        inside: value,
        outside: value,
    };
    match (classify_unit(outside), classify_unit(inside)) {
        (CharacterKind::Other, CharacterKind::Other) => both(marker == Some('_')),
        (CharacterKind::Other, CharacterKind::Whitespace) => both(true),
        (CharacterKind::Other, CharacterKind::Punctuation) => EncodeInfo {
            inside: false,
            outside: true,
        },
        (CharacterKind::Whitespace, CharacterKind::Whitespace) => both(true),
        (CharacterKind::Whitespace, _) => both(false),
        (CharacterKind::Punctuation, CharacterKind::Whitespace) => EncodeInfo {
            inside: true,
            outside: false,
        },
        (CharacterKind::Punctuation, _) => both(false),
    }
}

// A token still wrong after pairing; upstream copies its run's tokens into a list per
// token, quadratic in the run's length, so the run is shared and only additions are kept
struct Wrong {
    run: usize,
    extra: Vec<usize>,
}

fn check(tokens: &[Token], before: &str, after: &str) -> Option<Mistake> {
    let mut runs: Vec<Run> = Vec::new();
    let mut wrong: Vec<Option<Wrong>> = (0..tokens.len()).map(|_| None).collect();
    let mut run_of: Vec<Option<usize>> = vec![None; tokens.len()];
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].attention.is_none() {
            index += 1;
            continue;
        }
        let marker = tokens[index].value.chars().next();
        let strict = match marker {
            Some('*') => Some(false),
            Some('_') => Some(true),
            _ => None,
        };
        let mut end = index + 1;
        while end < tokens.len()
            && tokens[end].attention.is_some()
            && tokens[end].value.chars().next() == marker
        {
            end += 1;
        }
        let head = if index > 0 {
            &tokens[index - 1].value
        } else {
            before
        };
        let previous = classify_unit(head.chars().next_back());
        let next = classify_unit(
            if end < tokens.len() {
                &tokens[end].value
            } else {
                after
            }
            .chars()
            .next(),
        );
        let previous_some = previous != CharacterKind::Other;
        let next_some = next != CharacterKind::Other;
        let open = !next_some || (next == CharacterKind::Punctuation && previous_some);
        let close =
            !previous_some || (previous == CharacterKind::Punctuation && next_some);
        let mut run = Run {
            tokens: (index..end).collect(),
            markers: Vec::new(),
            start: 0,
            end: 0,
            open: if strict == Some(true) {
                open && (previous_some || !close)
            } else {
                open
            },
            close: if strict == Some(true) {
                close && (next_some || !open)
            } else {
                close
            },
            split: strict.is_some(),
        };
        for (token, each) in tokens.iter().enumerate().take(end).skip(index) {
            wrong[token] = Some(Wrong {
                run: runs.len(),
                extra: Vec::new(),
            });
            run_of[token] = Some(runs.len());
            run.markers
                .extend(std::iter::repeat_n(token, each.value.len()));
        }
        run.end = run.markers.len();
        runs.push(run);
        index = end;
    }
    pair(tokens, &mut runs, &mut wrong);
    let index = wrong.iter().position(Option::is_some)?;
    let first = wrong[index].as_ref()?;
    let involved = runs[first.run].tokens.iter().chain(&first.extra);
    Some(Mistake {
        attention: find_candidates(tokens, involved, &runs, &run_of),
        index,
    })
}

fn find_candidates<'t>(
    tokens: &[Token],
    involved: impl Iterator<Item = &'t usize>,
    runs: &[Run],
    run_of: &[Option<usize>],
) -> Vec<usize> {
    let involved: HashSet<usize> = involved
        .filter_map(|token| tokens[*token].attention)
        .collect();
    let mut related = HashSet::new();
    let mut visited = HashSet::new();
    for (index, token) in tokens.iter().enumerate() {
        if let (Some(run), Some(attention)) = (run_of[index], token.attention)
            && involved.contains(&attention)
            && visited.insert(run)
        {
            for sibling in &runs[run].tokens {
                if let Some(attention) = tokens[*sibling].attention {
                    related.insert(attention);
                }
            }
        }
    }
    let mut ordered = Vec::new();
    for token in tokens {
        if let Some(attention) = token.attention
            && related.remove(&attention)
        {
            ordered.push(attention);
        }
    }
    ordered.reverse();
    ordered
}

// The open runs of one marker, by what decides whether a closer may pair with them:
// scanning the stack past every skipped opener, as upstream does, is quadratic
#[derive(Default)]
struct Openers {
    split: [BTreeSet<usize>; 6],
    by_size: HashMap<usize, BTreeSet<usize>>,
}

impl Openers {
    fn insert(&mut self, runs: &[Run], run: usize) {
        if let Some(set) = self.find_set(runs, run) {
            set.insert(run);
        }
    }

    fn remove(&mut self, runs: &[Run], run: usize) {
        if let Some(set) = self.find_set(runs, run) {
            set.remove(&run);
        }
    }

    fn find_set(&mut self, runs: &[Run], run: usize) -> Option<&mut BTreeSet<usize>> {
        let size = runs[run].end - runs[run].start;
        if runs[run].split {
            self.split
                .get_mut(usize::from(runs[run].close) * 3 + size % 3)
        } else {
            Some(self.by_size.entry(size).or_default())
        }
    }

    // The topmost opener upstream's scan would not skip for `closer`
    fn find_opener(&self, runs: &[Run], closer: usize) -> Option<usize> {
        let size = runs[closer].end - runs[closer].start;
        if !runs[closer].split {
            return self.by_size.get(&size).and_then(|set| set.last().copied());
        }
        let mut best = None;
        for (class, set) in self.split.iter().enumerate() {
            let (close, remainder) = (class >= 3, class % 3);
            let skip = (close || runs[closer].open)
                && !size.is_multiple_of(3)
                && (remainder + size).is_multiple_of(3);
            if !skip {
                best = best.max(set.last().copied());
            }
        }
        best
    }
}

fn pair(tokens: &[Token], runs: &mut [Run], wrong: &mut [Option<Wrong>]) {
    let mut openers_by_marker: HashMap<Option<char>, Openers> = HashMap::new();
    for closer in 0..runs.len() {
        let marker = tokens[runs[closer].tokens[0]].value.chars().next();
        let openers = openers_by_marker.entry(marker).or_default();
        while runs[closer].close && runs[closer].start < runs[closer].end {
            let Some(opener) = openers.find_opener(runs, closer) else {
                break;
            };
            let opener_size = runs[opener].end - runs[opener].start;
            let closer_size = runs[closer].end - runs[closer].start;
            let size = if runs[closer].split {
                if opener_size > 1 && closer_size > 1 {
                    2
                } else {
                    1
                }
            } else {
                closer_size
            };
            let opening =
                &runs[opener].markers[runs[opener].end - size..runs[opener].end];
            let closing =
                &runs[closer].markers[runs[closer].start..runs[closer].start + size];
            let open_token = opening[0];
            let close_token = closing[0];
            if open_token == opening[size - 1]
                && tokens[open_token].value.len() == size
                && close_token == closing[size - 1]
                && tokens[close_token].value.len() == size
                && tokens[open_token].attention == tokens[close_token].attention
            {
                wrong[open_token] = None;
                wrong[close_token] = None;
            } else {
                let mut involved: Vec<usize> = Vec::new();
                for token in opening.iter().chain(closing) {
                    if !involved.contains(token) {
                        involved.push(*token);
                    }
                }
                for token in &involved {
                    if let Some(list) = wrong[*token].as_mut() {
                        list.extra.extend_from_slice(&involved);
                    }
                }
            }
            openers.remove(runs, opener);
            runs[opener].end -= size;
            runs[closer].start += size;
            if runs[opener].start < runs[opener].end {
                openers.insert(runs, opener);
            }
        }
        if runs[closer].open && runs[closer].start < runs[closer].end {
            openers.insert(runs, closer);
        }
    }
}

fn improve<'i>(
    items: &'i [Item],
    store: &'i [Attention],
    chosen: &Chosen,
    mistake: &Mistake,
    before: &str,
    after: &str,
    budget: &mut usize,
) -> Improvement<'i> {
    for attention in &mistake.attention {
        let current = chosen.get(attention).copied().unwrap_or(0);
        let count = store
            .get(*attention)
            .map_or(0, |attention| attention.sequences.len());
        for sequence in 0..count {
            if sequence == current {
                continue;
            }
            let mut trial = chosen.clone();
            trial.insert(*attention, sequence);
            let tokens = render_tokens(items, store, &trial, before, after);
            let Some(left) = budget.checked_sub(tokens.len() + trial.len()) else {
                return Improvement::OutOfBudget;
            };
            *budget = left;
            let next = check(&tokens, before, after);
            if next.as_ref().is_none_or(|next| next.index > mistake.index) {
                return Improvement::Found(trial, tokens, next);
            }
        }
    }
    Improvement::None
}
