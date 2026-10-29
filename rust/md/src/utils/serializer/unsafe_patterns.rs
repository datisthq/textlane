use regex::bytes::{Regex, RegexBuilder};
use textlane_core::Error;

use crate::models::options::{MathOptions, Names, UnsafePattern};
use crate::utils::serializer::construct::Construct;

/// An unsafe pattern, compiled as mdast-util-to-markdown's `compilePattern` does.
#[derive(Clone, Debug)]
pub(crate) struct Pattern {
    pub(crate) character: String,
    pub(crate) at_break: bool,
    /// `'before' in pattern || atBreak`: a match counts as preceded by something.
    pub(crate) has_before: bool,
    pub(crate) has_after: bool,
    /// Neither `before` nor `after` is given.
    pub(crate) is_bare: bool,
    pub(crate) in_construct: Option<Vec<Construct>>,
    pub(crate) not_in_construct: Option<Vec<Construct>>,
    pub(crate) before_node: Option<Vec<String>>,
    pub(crate) after_node: Option<Vec<String>>,
    pub(crate) expression: Regex,
}

const FULL_PHRASING_SPANS: &[&str] = &[
    "autolink",
    "destinationLiteral",
    "destinationRaw",
    "reference",
    "titleQuote",
    "titleApostrophe",
];

const CODE_FENCED_LANG: &[&str] = &["codeFencedLangGraveAccent", "codeFencedLangTilde"];

const CODE_FENCED_INFO: &[&str] = &[
    "codeFencedLangGraveAccent",
    "codeFencedLangTilde",
    "codeFencedMetaGraveAccent",
    "codeFencedMetaTilde",
    "destinationLiteral",
    "headingAtx",
];

/// The built-in patterns, GFM's when `gfm` is set, math's with `math` and directives' when
/// `directive` is, then `extra`, in the order upstream checks them.
pub(crate) fn compile_patterns(
    gfm: bool,
    math: Option<&MathOptions>,
    directive: bool,
    mdx: bool,
    extra: &[UnsafePattern],
) -> Result<Vec<Pattern>, Error> {
    let mut patterns = Vec::new();
    let gfm_patterns = if gfm { create_gfm_patterns() } else { vec![] };
    let math_patterns = math.map(create_math_patterns).unwrap_or_default();
    let directive_patterns = if directive {
        create_directive_patterns()
    } else {
        vec![]
    };
    let mdx_patterns = if mdx { create_mdx_patterns() } else { vec![] };
    for pattern in create_default_patterns()
        .iter()
        .chain(&gfm_patterns)
        .chain(&math_patterns)
        .chain(&directive_patterns)
        .chain(&mdx_patterns)
        .chain(extra)
    {
        patterns.push(compile_pattern(pattern)?);
    }
    Ok(patterns)
}

// In remark-gfm's order: autolink literals, footnotes, strikethrough, tables, task list
// items; an earlier pattern decides how a character is escaped
fn create_gfm_patterns() -> Vec<UnsafePattern> {
    let phrasing = Some(&["phrasing"][..]);
    let not_in_link = Some(&["autolink", "link", "image", "label"][..]);
    let table_cell = Some(&["tableCell"][..]);
    vec![
        create(
            '@',
            Some("[+\\-.\\w]"),
            Some("[\\-.\\w]"),
            phrasing,
            not_in_link,
        ),
        create('.', Some("[Ww]"), Some("[\\-.\\w]"), phrasing, not_in_link),
        create(':', Some("[ps]"), Some("\\/"), phrasing, not_in_link),
        create(
            '[',
            None,
            None,
            Some(&["label", "phrasing", "reference"]),
            None,
        ),
        create('~', None, None, phrasing, Some(FULL_PHRASING_SPANS)),
        create('\r', None, None, table_cell, None),
        create('\n', None, None, table_cell, None),
        create_at_break('|', None, Some("[\t :-]")),
        create('|', None, None, table_cell, None),
        create_at_break(':', None, Some("-")),
        create_at_break('-', None, Some("[:|-]")),
        create_at_break('-', None, Some("[:|-]")),
    ]
}

// As mdast-util-math: a single dollar is escaped in phrasing unless single dollars do not
// form math
fn create_math_patterns(options: &MathOptions) -> Vec<UnsafePattern> {
    let meta = Some(&["mathFlowMeta"][..]);
    let after = if options.single_dollar_text_math {
        None
    } else {
        Some("\\$")
    };
    vec![
        create('\r', None, None, meta, None),
        create('\n', None, None, meta, None),
        create('$', None, after, Some(&["phrasing"]), None),
        create('$', None, None, meta, None),
        create_at_break('$', None, Some("\\$")),
    ]
}

// mdast-util-directive's: line endings in flow labels, and colons that would start one
fn create_directive_patterns() -> Vec<UnsafePattern> {
    let labels = Some(&["leafDirectiveLabel", "containerDirectiveLabel"][..]);
    let phrasing = Some(&["phrasing"][..]);
    vec![
        create('\r', None, None, labels, None),
        create('\n', None, None, labels, None),
        UnsafePattern {
            unicode: true,
            ..create(
                ':',
                Some("[^:]"),
                Some("[^\\s\\p{P}\\p{S}]"),
                phrasing,
                None,
            )
        },
        UnsafePattern {
            after_node: Some(Names::One("textDirective".into())),
            ..create(':', None, None, phrasing, None)
        },
        create_at_break(':', Some(":+"), None),
    ]
}

// mdast-util-mdx's: `{` and `<` would start an expression or a tag
fn create_mdx_patterns() -> Vec<UnsafePattern> {
    let phrasing = Some(&["phrasing"][..]);
    vec![
        create('{', None, None, phrasing, None),
        create_at_break('{', None, None),
        create('<', None, None, phrasing, None),
        create_at_break('<', None, None),
    ]
}

fn create_default_patterns() -> Vec<UnsafePattern> {
    let phrasing = Some(&["phrasing"][..]);
    let mut patterns = vec![
        create('\t', None, Some("[\\r\\n]"), phrasing, None),
        create('\t', Some("[\\r\\n]"), None, phrasing, None),
        create('\t', None, None, Some(CODE_FENCED_LANG), None),
        create('\r', None, None, Some(CODE_FENCED_INFO), None),
        create('\n', None, None, Some(CODE_FENCED_INFO), None),
        create('\r', Some("[\\r\\n]"), None, phrasing, None),
        create('\r', None, Some("\\r"), phrasing, None),
        create('\n', Some("\\n"), None, phrasing, None),
        create('\n', None, Some("[\\r\\n]"), phrasing, None),
        create(' ', None, Some("[\\r\\n]"), phrasing, None),
        create(' ', Some("[\\r\\n]"), None, phrasing, None),
        create(' ', None, None, Some(CODE_FENCED_LANG), None),
        create('!', None, Some("\\["), phrasing, Some(FULL_PHRASING_SPANS)),
        create('"', None, None, Some(&["titleQuote"]), None),
        create_at_break('#', None, None),
        create('#', None, Some("(?:[\r\n]|$)"), Some(&["headingAtx"]), None),
        create(
            '&',
            None,
            Some("[#A-Za-z]"),
            Some(&[
                "codeFencedLangGraveAccent",
                "codeFencedLangTilde",
                "codeFencedMetaGraveAccent",
                "codeFencedMetaTilde",
                "destinationLiteral",
                "destinationRaw",
                "phrasing",
                "titleApostrophe",
                "titleQuote",
            ]),
            None,
        ),
        create('\'', None, None, Some(&["titleApostrophe"]), None),
        create('(', None, None, Some(&["destinationRaw"]), None),
        create('(', Some("\\]"), None, phrasing, Some(FULL_PHRASING_SPANS)),
        create_at_break(')', Some("\\d+"), None),
        create(')', None, None, Some(&["destinationRaw"]), None),
        create_at_break('*', None, Some("(?:[ \t\r\n*])")),
        create('*', None, None, phrasing, Some(FULL_PHRASING_SPANS)),
        create_at_break('+', None, Some("(?:[ \t\r\n])")),
        create_at_break('-', None, Some("(?:[ \t\r\n-])")),
        create_at_break('.', Some("\\d+"), Some("(?:[ \t\r\n]|$)")),
        create_at_break('<', None, Some("[!/?A-Za-z]")),
        create(
            '<',
            None,
            Some("[!/?A-Za-z]"),
            phrasing,
            Some(FULL_PHRASING_SPANS),
        ),
        create('<', None, None, Some(&["destinationLiteral"]), None),
        create_at_break('=', None, None),
        create_at_break('>', None, None),
        create('>', None, None, Some(&["destinationLiteral"]), None),
        create_at_break('[', None, None),
        create('[', None, None, phrasing, Some(FULL_PHRASING_SPANS)),
        create('[', None, None, Some(&["label", "reference"]), None),
        create('\\', None, Some("[\\r\\n]"), phrasing, None),
        create(']', None, None, Some(&["label", "reference"]), None),
        create_at_break('_', None, None),
        create('_', None, None, phrasing, Some(FULL_PHRASING_SPANS)),
        create_at_break('`', None, None),
        create(
            '`',
            None,
            None,
            Some(&["codeFencedLangGraveAccent", "codeFencedMetaGraveAccent"]),
            None,
        ),
        create('`', None, None, phrasing, Some(FULL_PHRASING_SPANS)),
        create_at_break('~', None, None),
        create('<', None, None, Some(&["autolink"]), None),
        create('>', None, None, Some(&["autolink"]), None),
        create('\u{7F}', None, None, Some(&["autolink"]), None),
    ];
    for code in 0..33u8 {
        patterns.push(create(
            char::from(code),
            None,
            None,
            Some(&["autolink"]),
            None,
        ));
    }
    patterns
}

/// mdast-util-definition-list's pattern: a `:` at a break would start a description.
pub(crate) fn create_def_list_pattern() -> UnsafePattern {
    create_at_break(':', None, Some("(?:[ \t\r\n])"))
}

fn create(
    character: char,
    before: Option<&str>,
    after: Option<&str>,
    in_construct: Option<&[&str]>,
    not_in_construct: Option<&[&str]>,
) -> UnsafePattern {
    let names =
        |list: &[&str]| Names::Many(list.iter().map(|name| name.to_string()).collect());
    UnsafePattern {
        character: character.to_string(),
        before: before.map(str::to_string),
        after: after.map(str::to_string),
        in_construct: in_construct.map(names),
        not_in_construct: not_in_construct.map(names),
        ..UnsafePattern::default()
    }
}

fn create_at_break(
    character: char,
    before: Option<&str>,
    after: Option<&str>,
) -> UnsafePattern {
    UnsafePattern {
        at_break: true,
        ..create(character, before, after, None, None)
    }
}

fn compile_pattern(pattern: &UnsafePattern) -> Result<Pattern, Error> {
    let mut before = String::new();
    if pattern.at_break {
        before.push_str("[\\r\\n][\\t ]*");
    }
    if let Some(value) = &pattern.before {
        before.push_str(&format!("(?:{value})"));
    }
    let mut source = String::new();
    if pattern.before_node.is_some() {
        source.push('^');
    } else if !before.is_empty() {
        source.push_str(&format!("({before})"));
    }
    for character in pattern.character.chars() {
        if character.is_ascii() {
            source.push_str(&format!("\\x{{{:X}}}", u32::from(character)));
        } else {
            source.push_str(&regex::escape(character.encode_utf8(&mut [0; 4])));
        }
    }
    if pattern.after_node.is_some() {
        source.push('$');
    } else if let Some(value) = &pattern.after {
        source.push_str(&format!("(?:{value})"));
    }
    let expression = RegexBuilder::new(&source)
        .unicode(pattern.unicode)
        .build()
        .map_err(|error| {
            Error::new(
                "invalid-options",
                format!("Cannot compile unsafe pattern `{source}`: {error}"),
            )
        })?;
    Ok(Pattern {
        character: pattern.character.clone(),
        at_break: pattern.at_break,
        has_before: pattern.before.is_some() || pattern.at_break,
        has_after: pattern.after.is_some(),
        is_bare: pattern.before.is_none() && pattern.after.is_none(),
        in_construct: read_constructs(pattern.in_construct.as_ref()),
        not_in_construct: read_constructs(pattern.not_in_construct.as_ref()),
        before_node: pattern
            .before_node
            .as_ref()
            .map(|names| names.as_slice().to_vec()),
        after_node: pattern
            .after_node
            .as_ref()
            .map(|names| names.as_slice().to_vec()),
        expression,
    })
}

// An empty list is no list; names of constructs that do not exist are never entered
fn read_constructs(names: Option<&Names>) -> Option<Vec<Construct>> {
    let names = names?.as_slice();
    if names.is_empty() {
        return None;
    }
    Some(
        names
            .iter()
            .filter_map(|name| Construct::from_name(name))
            .collect(),
    )
}
