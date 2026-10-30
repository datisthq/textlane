use textlane_core::mdast::{Link, List, ListItem, Node, Paragraph};
use textlane_core::{Allocator, ArenaVec};

use crate::plugins::toc::actions::heading::search::Entry;

/// How the table is written.
pub(crate) struct ContentsSettings<'s> {
    pub(crate) ordered: bool,
    pub(crate) tight: bool,
    pub(crate) prefix: &'s str,
}

/// mdast-util-toc's `contents`: the entries as nested lists, each depth one level below
/// the shallowest.
pub(crate) fn build_contents<'a>(
    map: Vec<Entry<'a>>,
    settings: &ContentsSettings<'_>,
    alloc: &'a Allocator,
) -> Node<'a> {
    let min_depth = map.iter().map(|entry| entry.depth).min().unwrap_or(1);
    // Lists and items by id; a child always comes after its parent
    let mut store = vec![Build::List {
        spread: false,
        items: Vec::new(),
    }];
    for (index, entry) in map.iter().enumerate() {
        insert_entry(
            &mut store,
            index,
            entry.depth - (min_depth - 1),
            settings.tight,
        );
    }
    let mut entries = map.into_iter().map(Some).collect::<Vec<_>>();
    let mut built: Vec<Option<Node<'a>>> = (0..store.len()).map(|_| None).collect();
    for id in (0..store.len()).rev() {
        let node = match &store[id] {
            Build::List { spread, items } => {
                let mut children = ArenaVec::with_capacity_in(items.len(), alloc);
                children.extend(items.iter().filter_map(|item| built[*item].take()));
                Node::List(List {
                    ordered: settings.ordered,
                    start: None,
                    spread: Some(*spread),
                    children,
                    position: None,
                    data: None,
                })
            }
            Build::Item {
                spread,
                entry,
                list,
            } => {
                let mut children = ArenaVec::new_in(alloc);
                if let Some(entry) = entry.and_then(|entry| entries[entry].take()) {
                    children.push(create_paragraph(entry, settings.prefix, alloc));
                }
                if let Some(list) = list.and_then(|list| built[list].take()) {
                    children.push(list);
                }
                Node::ListItem(ListItem {
                    spread: Some(*spread),
                    checked: None,
                    children,
                    position: None,
                    data: None,
                })
            }
        };
        built[id] = Some(node);
    }
    built.into_iter().next().flatten().unwrap_or_else(|| {
        Node::List(List {
            ordered: settings.ordered,
            start: None,
            spread: Some(false),
            children: ArenaVec::new_in(alloc),
            position: None,
            data: None,
        })
    })
}

enum Build {
    List {
        spread: bool,
        items: Vec<usize>,
    },
    // An item holds the link to its entry, if made for one, then a nested list
    Item {
        spread: bool,
        entry: Option<usize>,
        list: Option<usize>,
    },
}

// `insert`, walking down from the table and setting `spread` on the way back up
fn insert_entry(store: &mut Vec<Build>, index: usize, depth: u8, tight: bool) {
    let mut depth = depth;
    let mut current = 0;
    let mut path = Vec::new();
    loop {
        path.push(current);
        let next = store.len();
        match &mut store[current] {
            Build::List { items, .. } => {
                if depth == 1 {
                    items.push(next);
                    store.push(Build::Item {
                        spread: false,
                        entry: Some(index),
                        list: None,
                    });
                    break;
                }
                if let Some(last) = items.last() {
                    current = *last;
                } else {
                    items.push(next);
                    store.push(Build::Item {
                        spread: false,
                        entry: None,
                        list: None,
                    });
                    current = next;
                }
            }
            Build::Item { list, .. } => {
                depth -= 1;
                if let Some(list) = list {
                    current = *list;
                } else {
                    *list = Some(next);
                    store.push(Build::List {
                        spread: false,
                        items: Vec::new(),
                    });
                    current = next;
                }
            }
        }
    }
    for id in path.into_iter().rev() {
        let loose = match &store[id] {
            Build::List { items, .. } if !tight => items.iter().any(|item| {
                matches!(
                    store[*item],
                    Build::Item {
                        entry: Some(_),
                        list: Some(_),
                        ..
                    }
                )
            }),
            _ => !tight,
        };
        match &mut store[id] {
            Build::List { spread, .. } | Build::Item { spread, .. } => *spread = loose,
        }
    }
}

fn create_paragraph<'a>(
    entry: Entry<'a>,
    prefix: &str,
    alloc: &'a Allocator,
) -> Node<'a> {
    let url = alloc.alloc_str(&format!("#{prefix}{}", entry.id));
    let mut children = ArenaVec::with_capacity_in(1, alloc);
    children.push(Node::Link(Link {
        url,
        title: None,
        children: entry.children,
        position: None,
        data: None,
    }));
    Node::Paragraph(Paragraph {
        children,
        position: None,
        data: None,
    })
}
