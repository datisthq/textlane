use super::count_syllables;

// Counts syllable 5.0.1 gives, through each kind of rule: problematic words, hand-matched
// expressions, normalized characters and split words
#[test]
fn count_like_syllable() {
    let cases = [
        ("Café", 2),
        ("creation", 3),
        ("reapply", 3),
        ("atheist", 3),
        ("shredded", 2),
        ("hyperbole", 4),
        ("queue", 1),
        ("mccoy", 2),
        ("coaxing", 3),
        ("beautiful", 3),
        ("don’t", 1),
        ("12abc", 1),
        ("über straße", 4),
        ("people", 2),
    ];
    for (word, count) in cases {
        assert_eq!(count_syllables(word), count, "{word}");
    }
}
