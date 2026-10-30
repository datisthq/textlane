use crate::helpers::abbreviation::{is_elision_affix, is_elision_prefix};
use crate::helpers::classify::{is_numerical, is_word_symbol_inner};
use crate::utils::parser::cursor::{Cursor, Next, modify_children};
use crate::utils::parser::store::{Kind, Store};

/// parse-latin's sentence plugins over `sentence`, after parse-english's when `english`.
pub(crate) fn run_sentence_plugins(
    store: &mut Store<'_>,
    sentence: usize,
    english: bool,
) {
    if english {
        modify_children(store, sentence, |store, cursor, index| {
            merge_english_elision_exceptions(store, cursor, index);
            Next::Advance
        });
    }
    modify_children(store, sentence, merge_initial_word_symbol);
    modify_children(store, sentence, merge_final_word_symbol);
    modify_children(store, sentence, merge_inner_word_symbol);
    modify_children(store, sentence, merge_inner_word_slash);
    modify_children(store, sentence, merge_initialisms);
    patch_position(store, sentence);
}

/// Set a parent's missing start or end from its first or last child, as `patchPosition`.
pub(crate) fn patch_position(store: &mut Store<'_>, parent: usize) {
    let children = store.children(parent);
    let (Some(&first), Some(&last)) = (children.front(), children.back()) else {
        return;
    };
    if let Some(span) = store.items[first].position
        && store.items[parent]
            .position
            .is_none_or(|own| own.start.is_none())
    {
        store.items[parent].position.get_or_insert_default().start = span.start;
    }
    if let Some(span) = store.items[last].position
        && store.items[parent]
            .position
            .is_none_or(|own| own.end.is_none())
    {
        store.items[parent].position.get_or_insert_default().end = span.end;
    }
}

fn is_mark_of(store: &Store<'_>, id: usize, value: &str) -> bool {
    store.is_mark(id) && store.value(id) == value
}

// `&` joins the word after it when no word comes before
fn merge_initial_word_symbol(
    store: &mut Store<'_>,
    cursor: &mut Cursor,
    index: usize,
) -> Next {
    let Some(child) = cursor.get(index).filter(|id| is_mark_of(store, *id, "&")) else {
        return Next::Advance;
    };
    let previous_is_word =
        index > 0 && cursor.get(index - 1).is_some_and(|id| store.is_word(id));
    let Some(next) = cursor.get(index + 1).filter(|id| store.is_word(*id)) else {
        return Next::Advance;
    };
    if previous_is_word {
        return Next::Advance;
    }
    cursor.remove(index);
    store.children_mut(next).push_front(child);
    store.copy_start(next, child);
    Next::To(index as isize - 1)
}

// A `-` ending a word joins it
fn merge_final_word_symbol(
    store: &mut Store<'_>,
    cursor: &mut Cursor,
    index: usize,
) -> Next {
    let Some(child) = cursor.get(index).filter(|id| is_mark_of(store, *id, "-")) else {
        return Next::Advance;
    };
    if index == 0 || cursor.get(index + 1).is_some_and(|id| store.is_word(id)) {
        return Next::Advance;
    }
    let Some(previous) = cursor.get(index - 1).filter(|id| store.is_word(*id)) else {
        return Next::Advance;
    };
    cursor.remove(index);
    store.children_mut(previous).push_back(child);
    store.copy_end(previous, child);
    Next::To(index as isize)
}

// Words joined by inner marks (`it's`, `e-mail`) become one
fn merge_inner_word_symbol(
    store: &mut Store<'_>,
    cursor: &mut Cursor,
    index: usize,
) -> Next {
    let Some(child) = cursor.get(index) else {
        return Next::Advance;
    };
    if index == 0 || !store.is_mark(child) {
        return Next::Advance;
    }
    let Some(previous) = cursor.get(index - 1).filter(|id| store.is_word(*id)) else {
        return Next::Advance;
    };
    let mut position = index;
    let mut tokens = Vec::new();
    let mut queue = Vec::new();
    while let Some(sibling) = cursor.get(position) {
        if store.is_word(sibling) {
            tokens.append(&mut queue);
            tokens.extend(store.children(sibling).iter().copied());
        } else if store.is_mark(sibling) && is_word_symbol_inner(store.value(sibling)) {
            queue.push(sibling);
        } else {
            break;
        }
        position += 1;
    }
    let Some(&last) = tokens.last() else {
        return Next::Advance;
    };
    for _ in index..position - queue.len() {
        cursor.remove(index);
    }
    store.children_mut(previous).extend(tokens);
    store.copy_end(previous, last);
    Next::To(index as isize)
}

// Short words around a slash (`n/a`, `km/h`) become one
fn merge_inner_word_slash(
    store: &mut Store<'_>,
    cursor: &mut Cursor,
    index: usize,
) -> Next {
    let Some(child) = cursor.get(index).filter(|id| is_mark_of(store, *id, "/")) else {
        return Next::Advance;
    };
    let Some(previous) = index
        .checked_sub(1)
        .and_then(|before| cursor.get(before))
        .filter(|id| store.is_word(*id))
    else {
        return Next::Advance;
    };
    let next = cursor.get(index + 1).filter(|id| store.is_word(*id));
    if !store.is_shorter(previous, 3)
        || next.is_some_and(|next| !store.is_shorter(next, 3))
    {
        return Next::Advance;
    }
    cursor.remove(index);
    store.children_mut(previous).push_back(child);
    let mut tail = child;
    if let Some(next) = next {
        cursor.remove(index);
        store.append_children(previous, next);
        tail = next;
    }
    store.copy_end(previous, tail);
    Next::To(index as isize)
}

// A full stop after an initialism (`e.g`, `U.S`) joins it, unless all its letters are digits
fn merge_initialisms(store: &mut Store<'_>, cursor: &mut Cursor, index: usize) -> Next {
    let Some(child) = cursor.get(index) else {
        return Next::Advance;
    };
    if index == 0 || store.kind(child) != Kind::Punctuation || store.value(child) != "." {
        return Next::Advance;
    }
    let Some(previous) = cursor.get(index - 1).filter(|id| store.is_word(*id)) else {
        return Next::Advance;
    };
    let length = store.children(previous).len();
    if length == 1 || length.is_multiple_of(2) {
        return Next::Advance;
    }
    let mut is_all_digits = true;
    for position in (0..length).rev() {
        let other = store.children(previous)[position];
        let value = store.short_text(other, 4);
        if position % 2 == 0 {
            match value {
                Some(value) if value.encode_utf16().count() <= 1 => {
                    if !is_numerical(&value) {
                        is_all_digits = false;
                    }
                }
                _ => return Next::Advance,
            }
        } else if value.as_deref() != Some(".") {
            if position < length - 2 {
                break;
            }
            return Next::Advance;
        }
    }
    if is_all_digits {
        return Next::Advance;
    }
    cursor.remove(index);
    store.children_mut(previous).push_back(child);
    store.copy_end(previous, child);
    Next::To(index as isize)
}

fn lower_text(store: &Store<'_>, id: usize) -> Option<String> {
    store.short_text(id, 64).map(|text| text.to_lowercase())
}

fn is_apostrophe(value: &str) -> bool {
    matches!(value, "'" | "\u{2019}")
}

// parse-english: `w/`, `ol'`, `'tis`, `'n'`; visited as unist-util-visit-children does, so
// the node moved into a removed one's place is skipped
fn merge_english_elision_exceptions(
    store: &mut Store<'_>,
    cursor: &mut Cursor,
    index: usize,
) {
    let Some(child) = cursor.get(index).filter(|id| store.is_mark(*id)) else {
        return;
    };
    let length = cursor.len();
    let value = store.value(child);
    let word_at = |store: &Store<'_>, at: Option<usize>| {
        at.and_then(|at| cursor.get(at))
            .filter(|id| store.is_word(*id))
    };
    if value == "/" {
        if let Some(sibling) = word_at(store, index.checked_sub(1))
            && lower_text(store, sibling).as_deref() == Some("w")
        {
            cursor.remove(index);
            store.children_mut(sibling).push_back(child);
            store.copy_end(sibling, child);
        }
        return;
    }
    if !is_apostrophe(value) {
        return;
    }
    let is_white_space = |at: usize| {
        cursor
            .get(at)
            .is_some_and(|id| store.kind(id) == Kind::WhiteSpace)
    };
    if index > 2
        && index < length - 1
        && let Some(sibling) = word_at(store, Some(index - 1))
        && is_white_space(index - 2)
        && is_white_space(index + 1)
        && lower_text(store, sibling).is_some_and(|text| is_elision_prefix(&text))
    {
        cursor.remove(index);
        store.children_mut(sibling).push_back(child);
        store.copy_end(sibling, child);
        return;
    }
    if index + 1 == length || (index > 0 && word_at(store, Some(index - 1)).is_some()) {
        return;
    }
    let Some(sibling) = word_at(store, Some(index + 1)) else {
        return;
    };
    let lower = lower_text(store, sibling);
    if lower.as_deref().is_some_and(is_elision_affix) {
        cursor.remove(index);
        store.children_mut(sibling).push_front(child);
        store.copy_start(sibling, child);
    } else if lower.as_deref() == Some("n")
        && let Some(other) = cursor.get(index + 2)
        && store.kind(other) == Kind::Punctuation
        && is_apostrophe(store.value(other))
    {
        cursor.remove(index);
        cursor.remove(index + 1);
        store.children_mut(sibling).push_front(child);
        store.children_mut(sibling).push_back(other);
        store.copy_start(sibling, child);
        store.copy_end(sibling, other);
    }
}
