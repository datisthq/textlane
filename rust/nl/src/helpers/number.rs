const LESS_THAN_TWENTY: [&str; 20] = [
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
];

const TENS: [&str; 10] = [
    "zero", "ten", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty",
    "ninety",
];

const SCALES: [u64; 5] = [
    1_000_000_000_000_000,
    1_000_000_000_000,
    1_000_000_000,
    1_000_000,
    1_000,
];

/// The largest number number-to-words spells: `Number.MAX_SAFE_INTEGER`.
pub(crate) const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// The first word number-to-words 1.2.5 spells `number` with: `eighty` for 81, `two` for
/// 2,000.
pub(crate) fn read_first_number_word(mut number: u64) -> &'static str {
    loop {
        if number < 20 {
            return LESS_THAN_TWENTY[number as usize];
        }
        if number < 100 {
            return TENS[(number / 10) as usize];
        }
        if number < 1_000 {
            number /= 100;
            continue;
        }
        number /= SCALES
            .iter()
            .find(|scale| number >= **scale)
            .copied()
            .unwrap_or(1_000);
    }
}
