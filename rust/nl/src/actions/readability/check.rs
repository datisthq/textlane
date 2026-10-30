use std::collections::HashSet;

use textlane_core::Allocator;
use textlane_core::nlcst::Node;

use crate::actions::nlcst::stringify::stringify_nlcst;
use crate::actions::nlcst::visit::visit_nlcst;
use crate::actions::syllable::count::count_syllables;
use crate::helpers::place::join_positions;
use crate::helpers::readability_table::{DALE_CHALL, SPACHE};
use crate::models::options::Readability;
use crate::models::report::Report;

const SOURCE: &str = "retext-readability";

#[derive(Default)]
struct Counts {
    word: f64,
    syllable: f64,
    letter: f64,
    polysyllabic: f64,
    complex_polysyllabic: f64,
    easy: f64,
    familiar: f64,
}

/// retext-readability's warnings: each sentence of enough words that most of seven
/// readability formulas find too hard for the audience's age.
pub(crate) fn check_readability<'a>(
    tree: &Node<'a>,
    options: &Readability,
    alloc: &'a Allocator,
) -> Vec<Report<'a>> {
    let age = options
        .age
        .filter(|age| *age != 0.0 && !age.is_nan())
        .unwrap_or(16.0);
    let threshold = options
        .threshold
        .filter(|threshold| *threshold != 0.0 && !threshold.is_nan())
        .unwrap_or(4.0 / 7.0);
    let min_words = options.min_words.unwrap_or(5.0);
    let mut messages = Vec::new();
    let is_sentence = |node: &Node<'_>| matches!(node, Node::Sentence(_));
    visit_nlcst(tree, is_sentence, true, |sentence| {
        let counts = count_sentence(sentence);
        if counts.word < min_words {
            return;
        }
        let scores = score_counts(&counts);
        let failing = scores.iter().filter(|score| **score > age).count();
        if (failing as f64 / scores.len() as f64) < threshold {
            return;
        }
        let according = if failing < scores.len() {
            format!("{failing} out of ")
        } else {
            "all ".to_string()
        };
        let reason = format!(
            "Unexpected hard to read sentence, according to {according}{} algorithms",
            scores.len()
        );
        let position = join_positions(sentence, sentence, alloc);
        messages.push(Report {
            reason,
            rule: "readability".to_string(),
            source: SOURCE,
            position,
        });
    });
    messages
}

fn count_sentence(sentence: &Node<'_>) -> Counts {
    let mut counts = Counts::default();
    let mut familiar = HashSet::new();
    let mut easy = HashSet::new();
    let is_word = |node: &Node<'_>| matches!(node, Node::Word(_));
    visit_nlcst(sentence, is_word, false, |word| {
        let value = stringify_nlcst(word);
        let caseless = value.to_lowercase();
        let syllables = count_syllables(&value);
        counts.word += 1.0;
        counts.syllable += syllables as f64;
        counts.letter += value.encode_utf16().count() as f64;
        if syllables >= 3 {
            counts.polysyllabic += 1.0;
            if value.encode_utf16().next() == caseless.encode_utf16().next() {
                counts.complex_polysyllabic += 1.0;
            }
        }
        if SPACHE.binary_search(&caseless.as_str()).is_ok()
            && familiar.insert(caseless.clone())
        {
            counts.familiar += 1.0;
        }
        if DALE_CHALL.binary_search(&caseless.as_str()).is_ok() && easy.insert(caseless) {
            counts.easy += 1.0;
        }
    });
    counts
}

// Each formula's grade as an age: automated-readability, coleman-liau, dale-chall-formula,
// flesch, gunning-fog, smog-formula and spache-formula, all over one sentence
fn score_counts(counts: &Counts) -> [f64; 7] {
    let words = counts.word;
    let nan_if_empty =
        |value: f64, needed: f64| if needed == 0.0 { f64::NAN } else { value };
    let automated = nan_if_empty(
        nan_if_empty(4.71 * (counts.letter / words) + 0.5 * words - 21.43, words),
        counts.letter,
    );
    let coleman = nan_if_empty(
        nan_if_empty(
            0.0588 * (counts.letter / words * 100.0)
                - 0.296 * (1.0 / words * 100.0)
                - 15.8,
            words,
        ),
        counts.letter,
    );
    let difficult = (words - counts.easy) / words;
    let mut dale = 0.1579 * difficult * 100.0 + 0.0496 * words;
    if difficult > 0.05 {
        dale += 3.6365;
    }
    let dale = nan_if_empty(dale, words);
    let flesch = nan_if_empty(
        nan_if_empty(
            206.835 - 1.015 * words - 84.6 * (counts.syllable / words),
            words,
        ),
        counts.syllable,
    );
    let fog = nan_if_empty(
        0.4 * (words + 100.0 * (counts.complex_polysyllabic / words)),
        words,
    );
    let smog = 3.1291 + 1.043 * (counts.polysyllabic * 30.0).sqrt();
    let spache = nan_if_empty(
        0.659 + 0.121 * words + 0.082 * ((words - counts.familiar) / words) * 100.0,
        words,
    );
    [
        grade_to_age(automated),
        grade_to_age(coleman),
        grade_to_age(dale_grade(dale)),
        20.0 - (flesch / 10.0).floor(),
        grade_to_age(fog),
        ((smog).sqrt() + 2.5).ceil(),
        grade_to_age(spache),
    ]
}

// dale-chall-formula's `daleChallGradeLevel`, the upper grade of the score's band
fn dale_grade(score: f64) -> f64 {
    if score.is_nan() {
        return f64::NAN;
    }
    match score.floor() {
        floored if floored < 5.0 => 4.0,
        floored if floored < 6.0 => 6.0,
        floored if floored < 7.0 => 8.0,
        floored if floored < 8.0 => 10.0,
        floored if floored < 9.0 => 12.0,
        floored if floored < 10.0 => 15.0,
        _ => f64::INFINITY,
    }
}

// `Math.round(grade + 5)`: halves round up, unlike Rust's `round`
fn grade_to_age(grade: f64) -> f64 {
    let value = grade + 5.0;
    let floor = value.floor();
    if value - floor >= 0.5 {
        floor + 1.0
    } else {
        floor
    }
}
