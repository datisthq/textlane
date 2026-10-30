use std::collections::HashMap;

use crate::helpers::normalize::normalize_word;

/// Phrases indexed by their first word, as nlcst-search groups them, so a word is tested only
/// against the phrases it can start.
#[derive(Clone, Debug, Default)]
pub(crate) struct Phrases {
    pub(crate) by_word: HashMap<String, Vec<usize>>,
    /// Phrases starting with `*`, which any word starts.
    pub(crate) any: Vec<usize>,
    /// Each phrase's later words, normalized; `None` is `*`.
    pub(crate) rest: Vec<Vec<Option<String>>>,
    pub(crate) allow_apostrophes: bool,
}

impl Phrases {
    pub(crate) fn new<'p>(
        phrases: impl IntoIterator<Item = &'p str>,
        allow_apostrophes: bool,
    ) -> Self {
        let mut index = Self {
            allow_apostrophes,
            ..Self::default()
        };
        for (position, phrase) in phrases.into_iter().enumerate() {
            let mut words = phrase.split(' ');
            let first =
                normalize_word(words.next().unwrap_or_default(), allow_apostrophes);
            if first == "*" {
                index.any.push(position);
            } else {
                index.by_word.entry(first).or_default().push(position);
            }
            index.rest.push(
                words
                    .map(|word| {
                        (word != "*").then(|| normalize_word(word, allow_apostrophes))
                    })
                    .collect(),
            );
        }
        index
    }
}
