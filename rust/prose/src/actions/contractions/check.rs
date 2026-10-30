use textlane_core::nlcst::Node;

use crate::actions::nlcst::stringify::stringify_nlcst;
use crate::helpers::contraction_table::CONTRACTIONS;
use crate::helpers::literal::is_literal;
use crate::helpers::words::visit_words;
use crate::models::contractions::Contractions;
use crate::models::report::Report;

/// Report contractions missing their apostrophe or using the other kind, as
/// retext-contractions does.
pub(crate) fn check_contractions<'a>(
    tree: &Node<'a>,
    options: &Contractions,
    reports: &mut Vec<Report<'a>>,
) {
    visit_words(tree, |node, place| {
        let actual = stringify_nlcst(node);
        let normal: String = actual
            .chars()
            .filter(|character| !matches!(character, '\'' | '’'))
            .collect();
        let Ok(found) =
            CONTRACTIONS.binary_search_by(|(word, _)| (*word).cmp(normal.as_str()))
        else {
            return;
        };
        let (_, expected) = CONTRACTIONS[found];
        let expected = if options.straight {
            expected.to_string()
        } else {
            expected.replace('\'', "’")
        };
        if actual == expected
            || (!options.allow_literals
                && is_literal(
                    place.siblings,
                    place.index,
                    Some((place.first_word, place.last_word)),
                ))
        {
            return;
        }
        let kind = if normal == actual {
            "missing"
        } else if options.straight {
            "smart"
        } else {
            "straight"
        };
        let style = if options.straight {
            "straight"
        } else {
            "smart"
        };
        reports.push(Report {
            reason: format!(
                "Unexpected {kind} apostrophe in `{actual}`, expected `{expected}`"
            ),
            rule: format!("missing-{style}-apostrophe"),
            source: "retext-contractions",
            position: node.position().copied(),
        });
    });
}
