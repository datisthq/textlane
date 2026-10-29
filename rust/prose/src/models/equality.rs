use crate::helpers::equality_table::PATTERNS;
use crate::models::options::EqualityOptions;
use crate::models::phrases::Phrases;

/// A phrase `prose.equality` searches: its pattern, its text and its category.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EqualityPhrase {
    pub pattern: usize,
    pub phrase: &'static str,
    pub kind: &'static str,
}

/// The phrases `prose.equality` searches, by whether apostrophes count, as retext-equality
/// splits them.
#[derive(Clone, Debug, Default)]
pub(crate) struct EqualityRules {
    pub binary: bool,
    pub lists: Vec<(Phrases, Vec<EqualityPhrase>)>,
}

impl EqualityRules {
    pub(crate) fn new(options: &EqualityOptions) -> Self {
        let ignore = options.ignore.as_deref().unwrap_or_default();
        let mut normal = Vec::new();
        let mut apostrophe = Vec::new();
        for (index, pattern) in PATTERNS.iter().enumerate() {
            for (phrase, kind) in pattern.inconsiderate {
                if ignore.iter().any(|ignored| ignored == phrase) {
                    continue;
                }
                let entry = EqualityPhrase {
                    pattern: index,
                    phrase,
                    kind,
                };
                if pattern.apostrophe {
                    apostrophe.push(entry);
                } else {
                    normal.push(entry);
                }
            }
        }
        let index = |list: Vec<EqualityPhrase>, allow_apostrophes: bool| {
            let phrases =
                Phrases::new(list.iter().map(|entry| entry.phrase), allow_apostrophes);
            (phrases, list)
        };
        Self {
            binary: options.binary.unwrap_or(false),
            lists: vec![index(normal, false), index(apostrophe, true)],
        }
    }
}
