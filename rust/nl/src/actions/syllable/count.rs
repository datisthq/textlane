use std::sync::LazyLock;

use regex::Regex;

use crate::actions::word::singularize::singularize_word;
use crate::helpers::readability_table::{CHARMAP, PROBLEMATIC};

struct Expressions {
    triple: Regex,
    double: Regex,
    single: Regex,
    monosyllabic_one: Regex,
    monosyllabic_two: Regex,
    double_syllabic_one: Regex,
    double_syllabic_three: Regex,
    double_syllabic_four: Regex,
}

// syllable 5.0.1's expressions; the two with a back reference or a lookahead, which `regex`
// lacks, are matched by hand (`ends_in_double_l`, `count_double_syllabic_two`)
static EXPRESSIONS: LazyLock<Option<Expressions>> = LazyLock::new(|| {
    let compile = |source: &str| Regex::new(source).ok();
    Some(Expressions {
        triple: compile("(creations?|ology|ologist|onomy|onomist)$")?,
        double: compile(concat!(
            "^(?:above|anti|ante|counter|hyper|afore|agri|infra|intra|inter|over|semi|ultra|",
            "under|extra|dia|micro|mega|kilo|pico|nano|macro|somer)|",
            "(?:fully|berry|woman|women|edly|union|((?:[bcdfghjklmnpqrstvwxz])|[aeiou])ye?ing)$",
        ))?,
        single: compile(concat!(
            "^(?:un|fore|ware|none?|out|post|sub|pre|pro|dis|side|some)|",
            "(?:ly|less|some|ful|ers?|ness|cians?|ments?|ettes?|villes?|ships?|sides?|ports?|",
            "shires?|[gnst]ion(?:ed|s)?)$",
        ))?,
        monosyllabic_one: compile(concat!(
            "awe($|d|so)|cia(?:l|$)|tia|cius|cious|[^aeiou]giu|[aeiouy][^aeiouy]ion|iou|sia$|",
            "eous$|[oa]gue$|.[^aeiuoycgltdb]{2,}ed$|.ely$|^jua|uai|eau|^busi$|",
            "(?:[aeiouy](?:[bcfgklmnprsvwxyz]|ch|dg|g[hn]|lch|l[lv]|mm|nch|n[cgn]|r[bcnsv]|squ|",
            "s[chkls]|th)ed$)|",
            "(?:[aeiouy](?:[bdfklmnprstvy]|ch|g[hn]|lch|l[lv]|mm|nch|nn|r[nsv]|squ|s[cklst]|",
            "th)es$)",
        ))?,
        monosyllabic_two: compile(concat!(
            "[aeiouy](?:[bcdfgklmnprstvyz]|ch|dg|g[hn]|l[lv]|mm|n[cgns]|r[cnsv]|squ|s[cklst]|",
            "th)e$",
        ))?,
        double_syllabic_one: compile(concat!(
            "(?:[^aeiouy]ie(?:r|s?t)|[aeiouym]bl|eo|ism|asm|thm|dnt|snt|uity|dea|gean|oa|ua|",
            "react?|orbed|shred|eings?|[aeiouy]sh?e[rs])$",
        ))?,
        double_syllabic_three: compile(concat!(
            "[^aeiou]y[ae]|[^l]lien|riet|dien|iu|io|ii|uen|[aeilotu]real|real[aeilotu]|iell|",
            "eo[^aeiou]|[aeiou]y[aeiou]",
        ))?,
        double_syllabic_four: compile("[^s]ia")?,
    })
});

/// The syllables in `value`, as syllable 5.0.1 counts them.
pub(crate) fn count_syllables(value: &str) -> i64 {
    let normalized: String = value
        .chars()
        .map(|character| {
            u16::try_from(u32::from(character))
                .ok()
                .and_then(|code| {
                    CHARMAP.binary_search_by_key(&code, |(key, _)| *key).ok()
                })
                .map_or_else(
                    || character.to_string(),
                    |index| CHARMAP[index].1.to_string(),
                )
        })
        .collect();
    let lowercase: String = normalized
        .to_lowercase()
        .chars()
        .filter(|character| !matches!(character, '\'' | '’'))
        .collect();
    // Split on (ASCII) word boundaries, then keep the letters of each word
    lowercase
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .map(|word| {
            let letters: String = word.chars().filter(char::is_ascii_lowercase).collect();
            count_word(&letters)
        })
        .sum()
}

fn count_word(value: &str) -> i64 {
    if value.is_empty() {
        return 0;
    }
    if value.len() < 3 {
        return 1;
    }
    if let Some(count) = find_problematic(value) {
        return count;
    }
    if let Some(count) = find_problematic(&singularize_word(value)) {
        return count;
    }
    let Some(expressions) = EXPRESSIONS.as_ref() else {
        return 1;
    };
    let mut count = 0;
    let mut value = value.to_string();
    for (expression, addition) in [
        (&expressions.triple, 3),
        (&expressions.double, 2),
        (&expressions.single, 1),
    ] {
        count += addition * expression.find_iter(&value).count() as i64;
        value = expression.replace_all(&value, "").into_owned();
    }
    count += value
        .split(|character: char| !"aeiouy".contains(character))
        .filter(|part| !part.is_empty())
        .count() as i64;
    count -= expressions.monosyllabic_one.find_iter(&value).count() as i64;
    count -= expressions.monosyllabic_two.find_iter(&value).count() as i64;
    let bytes = value.as_bytes();
    count += i64::from(
        expressions.double_syllabic_one.is_match(&value) || ends_in_double_l(bytes),
    );
    count += count_double_syllabic_two(bytes);
    count += expressions.double_syllabic_three.find_iter(&value).count() as i64;
    count += i64::from(expressions.double_syllabic_four.is_match(&value));
    if count == 0 { 1 } else { count }
}

fn find_problematic(value: &str) -> Option<i64> {
    let index = PROBLEMATIC
        .binary_search_by_key(&value, |(word, _)| word)
        .ok()?;
    i64::try_from(PROBLEMATIC[index].1).ok()
}

// `([^aeiouy])\1l$`
fn ends_in_double_l(bytes: &[u8]) -> bool {
    matches!(bytes, [.., first, second, b'l'] if first == second && !is_vowel(*first))
}

// `creat(?!u)|[^gq]ua[^auieo]|[aeiou]{3}|^(?:ia|mc|coa[dglx].)|^re(app|es|im|us)|(th|d)eist`,
// matched globally: at each place the first alternative that fits, then on past its end
fn count_double_syllabic_two(bytes: &[u8]) -> i64 {
    let mut count = 0;
    let mut index = 0;
    while index < bytes.len() {
        let rest = &bytes[index..];
        let length = if rest.starts_with(b"creat") && rest.get(5) != Some(&b'u') {
            Some(5)
        } else if let [first, b'u', b'a', last, ..] = rest
            && !matches!(first, b'g' | b'q')
            && !matches!(last, b'a' | b'u' | b'i' | b'e' | b'o')
        {
            Some(4)
        } else if rest.len() >= 3 && rest[..3].iter().all(|byte| b"aeiou".contains(byte))
        {
            Some(3)
        } else if index == 0 && (rest.starts_with(b"ia") || rest.starts_with(b"mc")) {
            Some(2)
        } else if index == 0
            && let [b'c', b'o', b'a', b'd' | b'g' | b'l' | b'x', _, ..] = rest
        {
            Some(5)
        } else if index == 0 && rest.starts_with(b"re") {
            [&b"app"[..], b"es", b"im", b"us"]
                .iter()
                .find(|tail| rest[2..].starts_with(tail))
                .map(|tail| 2 + tail.len())
        } else {
            None
        };
        let length = length.or_else(|| {
            if rest.starts_with(b"theist") {
                Some(6)
            } else if rest.starts_with(b"deist") {
                Some(5)
            } else {
                None
            }
        });
        match length {
            Some(length) => {
                count += 1;
                index += length;
            }
            None => index += 1,
        }
    }
    count
}

fn is_vowel(byte: u8) -> bool {
    b"aeiouy".contains(&byte)
}

#[cfg(test)]
#[path = "count_unit.rs"]
mod unit;
