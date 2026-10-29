use std::collections::VecDeque;

use crate::helpers::abbreviation::{is_english_abbreviation, is_latin_abbreviation};
use crate::helpers::classify::{
    count_line_endings, is_affix_symbol, is_lower_initial, is_terminal_marker,
};
use crate::utils::parser::cursor::{Cursor, Next, modify_children};
use crate::utils::parser::sentence::patch_position;
use crate::utils::parser::store::{Kind, Span, Store};

/// parse-latin's paragraph plugins over `paragraph`, after parse-english's when `english`.
pub(crate) fn run_paragraph_plugins(
    store: &mut Store<'_>,
    paragraph: usize,
    english: bool,
) {
    if english {
        modify_children(store, paragraph, merge_english_prefix_exceptions);
    }
    modify_children(store, paragraph, merge_non_word_sentences);
    modify_children(store, paragraph, merge_affix_symbol);
    modify_children(store, paragraph, merge_initial_lower_case_letter_sentences);
    modify_children(store, paragraph, merge_initial_digit_sentences);
    modify_children(store, paragraph, merge_prefix_exceptions);
    modify_children(store, paragraph, merge_affix_exceptions);
    let sentences: Vec<usize> = store.children(paragraph).iter().copied().collect();
    for sentence in sentences {
        merge_remaining_full_stops(store, sentence);
    }
    make_initial_white_space_siblings(store, paragraph);
    make_final_white_space_siblings(store, paragraph);
    modify_children(store, paragraph, break_implicit_sentences);
    remove_empty_nodes(store, paragraph);
    patch_position(store, paragraph);
}

/// parse-latin's root plugins over `root`.
pub(crate) fn run_root_plugins(store: &mut Store<'_>, root: usize) {
    make_initial_white_space_siblings(store, root);
    make_final_white_space_siblings(store, root);
    remove_empty_nodes(store, root);
    patch_position(store, root);
}

/// Split `parent`'s children into parents of its kind, each ending after a child of `kind`
/// that `test` accepts, or at the last child, as parse-latin's `splitNode`.
pub(crate) fn split_node(
    store: &mut Store<'_>,
    parent: usize,
    kind: Kind,
    test: fn(&str) -> bool,
) -> VecDeque<usize> {
    let parent_kind = store.kind(parent);
    let children = std::mem::take(store.children_mut(parent));
    let length = children.len();
    let mut result = VecDeque::new();
    let mut group = VecDeque::new();
    for (index, child) in children.into_iter().enumerate() {
        group.push_back(child);
        if index + 1 == length || (store.kind(child) == kind && test(store.value(child)))
        {
            let first = group.front().copied().unwrap_or(child);
            let position =
                match (store.items[first].position, store.items[child].position) {
                    (Some(first), Some(last)) => Some(Span {
                        start: first.start,
                        end: last.end,
                    }),
                    _ => None,
                };
            result.push_back(store.add_parent(
                parent_kind,
                std::mem::take(&mut group),
                position,
            ));
        }
    }
    result
}

fn has_children(store: &Store<'_>, id: usize) -> bool {
    store.is_parent(id) && !store.children(id).is_empty()
}

fn is_sentence(store: &Store<'_>, id: usize) -> bool {
    store.kind(id) == Kind::Sentence
}

// Move `child`'s children to the end of `previous`, which takes over its end, and drop it
fn merge_into_previous(
    store: &mut Store<'_>,
    cursor: &mut Cursor,
    index: usize,
    previous: usize,
) {
    if let Some(child) = cursor.get(index) {
        store.append_children(previous, child);
        store.copy_end(previous, child);
        cursor.remove(index);
    }
}

// A sentence ending in an abbreviation followed by a full stop continues in the next one
fn merge_period_into_abbreviation(
    store: &mut Store<'_>,
    cursor: &mut Cursor,
    index: usize,
    is_abbreviation: fn(&Store<'_>, usize) -> bool,
) -> Next {
    let Some(child) = cursor.get(index).filter(|id| store.is_parent(*id)) else {
        return Next::Advance;
    };
    let children = store.children(child);
    let length = children.len();
    let (Some(&period), Some(&word)) = (
        children.back(),
        length.checked_sub(2).and_then(|at| children.get(at)),
    ) else {
        return Next::Advance;
    };
    if !store.is_word(word) || !is_abbreviation(store, word) {
        return Next::Advance;
    }
    store.children_mut(word).push_back(period);
    store.children_mut(child).pop_back();
    store.copy_end(word, period);
    if let Some(next) = cursor.get(index + 1).filter(|id| is_sentence(store, *id)) {
        store.append_children(child, next);
        cursor.remove(index + 1);
        store.copy_end(child, next);
        return Next::To(index as isize - 1);
    }
    Next::Advance
}

fn merge_english_prefix_exceptions(
    store: &mut Store<'_>,
    cursor: &mut Cursor,
    index: usize,
) -> Next {
    let is_period = |store: &Store<'_>, id: usize| {
        store.kind(id) == Kind::Punctuation && store.value(id) == "."
    };
    match cursor
        .get(index)
        .and_then(|id| store.children(id).back().copied())
    {
        Some(period) if is_period(store, period) => {}
        _ => return Next::Advance,
    }
    merge_period_into_abbreviation(store, cursor, index, |store, word| {
        store
            .short_text(word, 64)
            .is_some_and(|value| is_english_abbreviation(&value, &value.to_lowercase()))
    })
}

fn merge_prefix_exceptions(
    store: &mut Store<'_>,
    cursor: &mut Cursor,
    index: usize,
) -> Next {
    match cursor
        .get(index)
        .and_then(|id| store.children(id).back().copied())
    {
        Some(period) if store.is_mark(period) && store.value(period) == "." => {}
        _ => return Next::Advance,
    }
    merge_period_into_abbreviation(store, cursor, index, |store, word| {
        store
            .short_text(word, 64)
            .is_some_and(|value| is_latin_abbreviation(&value.to_lowercase()))
    })
}

// A sentence without words joins the one before it, or else the one after
fn merge_non_word_sentences(
    store: &mut Store<'_>,
    cursor: &mut Cursor,
    index: usize,
) -> Next {
    let Some(child) = cursor.get(index).filter(|id| store.is_parent(*id)) else {
        return Next::Advance;
    };
    if store.children(child).iter().any(|id| store.is_word(*id)) {
        return Next::Advance;
    }
    if let Some(previous) = index
        .checked_sub(1)
        .and_then(|before| cursor.get(before))
        .filter(|id| store.is_parent(*id))
    {
        merge_into_previous(store, cursor, index, previous);
        return Next::To(index as isize);
    }
    if let Some(next) = cursor.get(index + 1).filter(|id| store.is_parent(*id)) {
        let moved = std::mem::take(store.children_mut(child));
        let children = store.children_mut(next);
        for id in moved.into_iter().rev() {
            children.push_front(id);
        }
        store.copy_start(next, child);
        cursor.remove(index);
    }
    Next::Advance
}

// Closing marks after a terminal marker (`."`, `?)`) belong to the sentence before
fn merge_affix_symbol(store: &mut Store<'_>, cursor: &mut Cursor, index: usize) -> Next {
    let Some(child) = cursor.get(index).filter(|id| has_children(store, *id)) else {
        return Next::Advance;
    };
    let Some(previous) = index
        .checked_sub(1)
        .and_then(|before| cursor.get(before))
        .filter(|id| is_sentence(store, *id))
    else {
        return Next::Advance;
    };
    let Some(&first) = store.children(child).front() else {
        return Next::Advance;
    };
    if !store.is_mark(first) || !is_affix_symbol(store.value(first)) {
        return Next::Advance;
    }
    store.children_mut(child).pop_front();
    store.children_mut(previous).push_back(first);
    store.copy_end(previous, first);
    if let Some(&second) = store.children(child).front() {
        store.copy_start(child, second);
    }
    Next::To(index as isize - 1)
}

// A sentence whose first word starts lower case continues the one before
fn merge_initial_lower_case_letter_sentences(
    store: &mut Store<'_>,
    cursor: &mut Cursor,
    index: usize,
) -> Next {
    let Some(child) = cursor.get(index).filter(|id| has_children(store, *id)) else {
        return Next::Advance;
    };
    let Some(previous) = index
        .checked_sub(1)
        .and_then(|before| cursor.get(before))
        .filter(|id| is_sentence(store, *id))
    else {
        return Next::Advance;
    };
    if !is_sentence(store, child) {
        return Next::Advance;
    }
    for &node in store.children(child) {
        if store.is_word(node) {
            if !store.first_character(node).is_some_and(is_lower_initial) {
                return Next::Advance;
            }
            merge_into_previous(store, cursor, index, previous);
            return Next::To(index as isize);
        }
        if store.is_mark(node) {
            return Next::Advance;
        }
    }
    Next::Advance
}

// A sentence starting with a digit continues the one before (`1.5`, `the 2. place`)
fn merge_initial_digit_sentences(
    store: &mut Store<'_>,
    cursor: &mut Cursor,
    index: usize,
) -> Next {
    let Some(child) = cursor.get(index).filter(|id| is_sentence(store, *id)) else {
        return Next::Advance;
    };
    let Some(previous) = index
        .checked_sub(1)
        .and_then(|before| cursor.get(before))
        .filter(|id| is_sentence(store, *id))
    else {
        return Next::Advance;
    };
    let Some(&head) = store.children(child).front() else {
        return Next::Advance;
    };
    if store.is_word(head)
        && store
            .first_character(head)
            .is_some_and(|character| character.is_ascii_digit())
    {
        merge_into_previous(store, cursor, index, previous);
        return Next::To(index as isize);
    }
    Next::Advance
}

// A sentence starting with a comma or semicolon continues the one before
fn merge_affix_exceptions(
    store: &mut Store<'_>,
    cursor: &mut Cursor,
    index: usize,
) -> Next {
    let Some(child) = cursor.get(index).filter(|id| has_children(store, *id)) else {
        return Next::Advance;
    };
    let Some(previous) = index
        .checked_sub(1)
        .and_then(|before| cursor.get(before))
        .filter(|id| store.is_parent(*id))
    else {
        return Next::Advance;
    };
    for &node in store.children(child) {
        if store.is_word(node) {
            return Next::Advance;
        }
        if store.is_mark(node) {
            if !matches!(store.value(node), "," | ";") {
                return Next::Advance;
            }
            merge_into_previous(store, cursor, index, previous);
            return Next::To(index as isize);
        }
    }
    Next::Advance
}

// Full stops that end no sentence join the word before them, or else the word after; walked
// from the end with a gap, as each one is spliced out where it stands
fn merge_remaining_full_stops(store: &mut Store<'_>, parent: usize) {
    if !store.is_parent(parent) {
        return;
    }
    let mut before: Vec<usize> = std::mem::take(store.children_mut(parent)).into();
    // Reversed: its last item is the node after the one being visited
    let mut after: Vec<usize> = Vec::new();
    let mut has_found_delimiter = false;
    while let Some(&grandchild) = before.last() {
        let skip = |before: &mut Vec<usize>, after: &mut Vec<usize>| {
            if let Some(id) = before.pop() {
                after.push(id);
            }
        };
        if !store.is_mark(grandchild) {
            if store.is_word(grandchild) {
                has_found_delimiter = true;
            }
            skip(&mut before, &mut after);
            continue;
        }
        let value = store.value(grandchild);
        if !is_terminal_marker(value) || !has_found_delimiter || value != "." {
            // The first terminal marker from the end ends the sentence and stays
            has_found_delimiter |= is_terminal_marker(value);
            skip(&mut before, &mut after);
            continue;
        }
        let previous = before.len().checked_sub(2).map(|at| before[at]);
        let next = after.last().copied();
        if let Some(previous) = previous.filter(|id| store.is_word(*id)) {
            let next_next = after.len().checked_sub(2).map(|at| after[at]);
            if let (Some(next), Some(next_next)) = (next, next_next)
                && store.kind(next) == Kind::WhiteSpace
                && store.short_text(next_next, 1).as_deref() == Some(".")
            {
                skip(&mut before, &mut after);
                continue;
            }
            before.pop();
            store.children_mut(previous).push_back(grandchild);
            store.copy_end(previous, grandchild);
            // parse-latin steps back twice: the word it joined is not visited
            skip(&mut before, &mut after);
        } else if let Some(next) = next.filter(|id| store.is_word(*id)) {
            before.pop();
            store.children_mut(next).push_front(grandchild);
            store.copy_start(next, grandchild);
        } else {
            skip(&mut before, &mut after);
        }
    }
    after.reverse();
    *store.children_mut(parent) = after.into();
}

// White space starting a child moves up before it; the child is visited again
fn make_initial_white_space_siblings(store: &mut Store<'_>, parent: usize) {
    modify_children(store, parent, |store, cursor, index| {
        let Some(child) = cursor.get(index).filter(|id| store.is_parent(*id)) else {
            return Next::Advance;
        };
        let Some(&head) = store.children(child).front() else {
            return Next::Advance;
        };
        if store.kind(head) == Kind::WhiteSpace {
            store.children_mut(child).pop_front();
            cursor.insert(index, head);
            if let Some(&next) = store.children(child).front() {
                store.copy_start(child, next);
            }
        }
        Next::Advance
    });
}

// White space ending a child moves up after it
fn make_final_white_space_siblings(store: &mut Store<'_>, parent: usize) {
    modify_children(store, parent, |store, cursor, index| {
        let Some(child) = cursor.get(index).filter(|id| store.is_parent(*id)) else {
            return Next::Advance;
        };
        let Some(&tail) = store.children(child).back() else {
            return Next::Advance;
        };
        if store.kind(tail) != Kind::WhiteSpace {
            return Next::Advance;
        }
        store.children_mut(child).pop_back();
        cursor.insert(index + 1, tail);
        if let Some(&previous) = store.children(child).back() {
            store.copy_end(child, previous);
        }
        Next::To(index as isize)
    });
}

// White space with two or more line endings inside a sentence ends it
fn break_implicit_sentences(
    store: &mut Store<'_>,
    cursor: &mut Cursor,
    index: usize,
) -> Next {
    let Some(child) = cursor.get(index).filter(|id| is_sentence(store, *id)) else {
        return Next::Advance;
    };
    let children = store.children(child);
    let Some(position) = (1..children.len().saturating_sub(1)).find(|&position| {
        let node = children[position];
        store.kind(node) == Kind::WhiteSpace && count_line_endings(store.value(node)) >= 2
    }) else {
        return Next::Advance;
    };
    let tail = children[position - 1];
    let head = children[position + 1];
    let mut rest = std::mem::take(store.children_mut(child));
    let kept: VecDeque<usize> = rest.drain(..position).collect();
    let Some(node) = rest.pop_front() else {
        return Next::Advance;
    };
    *store.children_mut(child) = kept;
    let insertion = store.add_parent(Kind::Sentence, rest, None);
    cursor.insert(index + 1, node);
    cursor.insert(index + 2, insertion);
    if let (Some(own), Some(_), Some(_)) = (
        store.items[child].position,
        store.items[tail].position,
        store.items[head].position,
    ) {
        store.copy_end(child, tail);
        store.items[insertion].position = Some(Span {
            start: store.items[head].position.and_then(|span| span.start),
            end: own.end,
        });
    }
    Next::To(index as isize + 1)
}

fn remove_empty_nodes(store: &mut Store<'_>, parent: usize) {
    modify_children(store, parent, |store, cursor, index| {
        match cursor.get(index) {
            Some(child) if store.is_parent(child) && store.children(child).is_empty() => {
                cursor.remove(index);
                Next::To(index as isize)
            }
            _ => Next::Advance,
        }
    });
}
