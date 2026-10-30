use std::collections::HashMap;

use crate::helpers::slug_table::REMOVED;

/// github-slugger's `BananaSlug`: slugs, made unique by counting repeats.
#[derive(Debug, Default)]
pub struct Slugger {
    occurrences: HashMap<String, usize>,
}

impl Slugger {
    /// A unique slug for `value`: `a`, then `a-1`, `a-2`.
    pub fn slug(&mut self, value: &str) -> String {
        let original = create_slug(value);
        let mut result = original.clone();
        while self.occurrences.contains_key(&result) {
            let count = self.occurrences.entry(original.clone()).or_default();
            *count += 1;
            result = format!("{original}-{count}");
        }
        self.occurrences.insert(result.clone(), 0);
        result
    }
}

/// Lowercase, drop punctuation and symbols, turn spaces into dashes.
pub fn create_slug(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .filter(|character| !is_removed(*character))
        .map(|character| if character == ' ' { '-' } else { character })
        .collect()
}

fn is_removed(character: char) -> bool {
    let code = u32::from(character);
    REMOVED
        .binary_search_by(|(start, end)| {
            if *end < code {
                std::cmp::Ordering::Less
            } else if *start > code {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}
