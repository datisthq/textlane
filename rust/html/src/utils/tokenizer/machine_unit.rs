// Expected streams are parse5 7.3.0's, with its UTF-16 locations turned into byte offsets
use super::{State, Tokenizer};
use crate::helpers::tag::TagId;
use crate::models::token::{CharacterKind, Span, TagToken, Token};

fn escape(text: &str) -> String {
    text.chars()
        .map(|char| match char {
            '"' | '\\' | '\0'..='\x1f' | '\x7f'.. => {
                format!("\\u{{{:x}}}", u32::from(char))
            }
            _ => char.to_string(),
        })
        .collect()
}

fn quote(text: Option<&str>) -> String {
    text.map_or_else(|| "null".to_owned(), |text| format!("\"{}\"", escape(text)))
}

fn format_span(span: Span) -> String {
    format!("{}..{}", span.start, span.end)
}

fn format_tag(kind: &str, tag: &TagToken) -> String {
    let mut line = format!(
        "{kind} {} {}",
        quote(Some(&tag.tag_name)),
        format_span(tag.span)
    );
    if tag.self_closing {
        line.push_str(" /");
    }
    for attribute in &tag.attrs {
        line.push_str(&format!(
            " {}={}@{}",
            quote(Some(&attribute.name)),
            quote(Some(&attribute.value)),
            format_span(attribute.span)
        ));
    }
    line
}

fn format_token(token: &Token) -> String {
    match token {
        Token::Character(character) => {
            let kind = match character.kind {
                CharacterKind::Character => "Character",
                CharacterKind::Null => "Null",
                CharacterKind::Whitespace => "Whitespace",
            };
            let chars = quote(Some(&character.chars));
            format!("{kind} {chars} {}", format_span(character.span))
        }
        Token::StartTag(tag) => format_tag("StartTag", tag),
        Token::EndTag(tag) => format_tag("EndTag", tag),
        Token::Comment(comment) => {
            format!(
                "Comment {} {}",
                quote(Some(&comment.data)),
                format_span(comment.span)
            )
        }
        Token::Doctype(doctype) => format!(
            "Doctype {} {} {} {}{}",
            quote(doctype.name.as_deref()),
            quote(doctype.public_id.as_deref()),
            quote(doctype.system_id.as_deref()),
            format_span(doctype.span),
            if doctype.force_quirks { " quirks" } else { "" }
        ),
        Token::Eof(span) => format!("Eof {}", format_span(*span)),
    }
}

/// Tokenizes `input`, switching states on start tags as parse5's tree builder does.
fn dump(input: &str, initial: Option<(State, &str)>) -> String {
    let mut tokenizer = Tokenizer::new(input);
    if let Some((state, last_start_tag_name)) = initial {
        tokenizer.state = state;
        last_start_tag_name.clone_into(&mut tokenizer.last_start_tag_name);
    }
    let mut lines = Vec::new();
    loop {
        let token = tokenizer.next_token();
        lines.push(format_token(&token));
        match &token {
            Token::StartTag(tag) => {
                tokenizer.state = match tag.tag_name.as_str() {
                    "title" | "textarea" => State::Rcdata,
                    "style" | "xmp" | "iframe" | "noembed" | "noframes" => State::Rawtext,
                    "script" => State::ScriptData,
                    "plaintext" => State::Plaintext,
                    _ => tokenizer.state,
                };
                if matches!(tag.tag_name.as_str(), "svg" | "math") {
                    tokenizer.in_foreign_node = true;
                }
            }
            Token::EndTag(tag) if matches!(tag.tag_name.as_str(), "svg" | "math") => {
                tokenizer.in_foreign_node = false;
            }
            Token::Eof(_) => break,
            _ => {}
        }
    }
    for error in &tokenizer.errors {
        lines.push(format!(
            "! {} {}..{}",
            error.code.as_str(),
            error.start,
            error.end
        ));
    }
    lines.join("\n")
}

fn report(input: &str, expected: &str, actual: &str) -> Option<String> {
    (actual != expected).then(|| {
        format!("input {input:?}\n--- expected\n{expected}\n--- actual\n{actual}")
    })
}

fn check(cases: &[(&str, &str)]) {
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|(input, expected)| report(input, expected, &dump(input, None)))
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

fn check_initial(cases: &[(&str, State, &str, &str)]) {
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|(input, state, last, expected)| {
            report(input, expected, &dump(input, Some((*state, last))))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn keeps_returning_eof_after_the_end() {
    let mut tokenizer = Tokenizer::new("a");
    let tokens: Vec<String> = (0..4)
        .map(|_| format_token(&tokenizer.next_token()))
        .collect();
    assert_eq!(
        tokens,
        ["Character \"a\" 0..1", "Eof 1..1", "Eof 1..1", "Eof 1..1"]
    );
}

#[test]
fn identifies_tags_by_their_lowercased_name() {
    let mut tokenizer = Tokenizer::new("<DIV></foreignObject><x-y>");
    let ids: Vec<TagId> = (0..3)
        .filter_map(|_| match tokenizer.next_token() {
            Token::StartTag(tag) | Token::EndTag(tag) => Some(tag.tag_id),
            _ => None,
        })
        .collect();
    assert_eq!(ids, [TagId::Div, TagId::Unknown, TagId::Unknown]);
}

#[test]
fn stops_after_each_token_so_the_state_can_switch() {
    let mut tokenizer = Tokenizer::new("<p>a<b></p>");
    tokenizer.next_token();
    tokenizer.state = State::Rawtext;
    tokenizer.last_start_tag_name = "p".to_owned();
    let tokens: Vec<String> = (0..3)
        .map(|_| format_token(&tokenizer.next_token()))
        .collect();
    assert_eq!(
        tokens,
        [
            "Character \"a<b>\" 3..7",
            "EndTag \"p\" 7..11",
            "Eof 11..11"
        ]
    );
}

#[test]
fn runs_in_linear_time_on_hostile_input() {
    let inputs = [
        "<a ".repeat(50_000),
        format!("<a {}>", "b ".repeat(50_000)),
        format!(
            "<a {}>",
            (0..50_000).map(|n| format!("a{n} ")).collect::<String>()
        ),
        "&".repeat(100_000),
        "&amp".repeat(50_000),
        "&notin".repeat(30_000),
        format!("&#{}", "9".repeat(100_000)),
        format!("<!--{}", "-".repeat(100_000)),
        format!("<!DOCTYPE {}", "a".repeat(100_000)),
        "</".repeat(50_000),
        format!("<script>{}", "<!--<script>".repeat(20_000)),
        format!("<title>{}", "</ti".repeat(30_000)),
    ];
    for input in &inputs {
        let mut tokenizer = Tokenizer::new(input);
        let mut count = 0;
        while !matches!(tokenizer.next_token(), Token::Eof(_)) {
            count += 1;
            if count > input.len() * 2 {
                break;
            }
        }
        assert!(count <= input.len() * 2);
    }
}

#[test]
fn tokenizes_text_and_whitespace_as_parse5() {
    check(&[
        ("", r#"Eof 0..0"#),
        (
            "abc",
            r#"Character "abc" 0..3
Eof 3..3"#,
        ),
        (
            " \u{9}\u{a}\u{c}",
            r#"Whitespace " \u{9}\u{a}\u{c}" 0..4
Eof 4..4"#,
        ),
        (
            "a b\u{9}c\u{a}",
            r#"Character "a" 0..1
Whitespace " " 1..2
Character "b" 2..3
Whitespace "\u{9}" 3..4
Character "c" 4..5
Whitespace "\u{a}" 5..6
Eof 6..6"#,
        ),
        (
            "  abc  ",
            r#"Whitespace "  " 0..2
Character "abc" 2..5
Whitespace "  " 5..7
Eof 7..7"#,
        ),
        (
            "a\u{0}b",
            r#"Character "a" 0..1
Null "\u{0}" 1..2
Character "b" 2..3
Eof 3..3
! unexpected-null-character 1..1"#,
        ),
        (
            "\u{0}\u{0} \u{0}x",
            r#"Null "\u{0}\u{0}" 0..2
Whitespace " " 2..3
Null "\u{0}" 3..4
Character "x" 4..5
Eof 5..5
! unexpected-null-character 0..0
! unexpected-null-character 1..1
! unexpected-null-character 3..3"#,
        ),
        (
            "a\u{d}\u{a}b\u{d}c\u{a}\u{d}d\u{d}\u{d}\u{a}",
            r#"Character "a" 0..1
Whitespace "\u{a}" 1..3
Character "b" 3..4
Whitespace "\u{a}" 4..5
Character "c" 5..6
Whitespace "\u{a}\u{a}" 6..8
Character "d" 8..9
Whitespace "\u{a}\u{a}" 9..12
Eof 12..12"#,
        ),
        (
            "\u{d}",
            r#"Whitespace "\u{a}" 0..1
Eof 1..1"#,
        ),
        (
            "\u{d}\u{a}",
            r#"Whitespace "\u{a}" 0..2
Eof 2..2"#,
        ),
        (
            "\u{a}\u{d}\u{a}",
            r#"Whitespace "\u{a}\u{a}" 0..3
Eof 3..3"#,
        ),
        (
            "\u{e9}\u{1f600}\u{fc}",
            r#"Character "\u{e9}\u{1f600}\u{fc}" 0..8
Eof 8..8"#,
        ),
        (
            " \u{1f600} \u{e9} ",
            r#"Whitespace " " 0..1
Character "\u{1f600}" 1..5
Whitespace " " 5..6
Character "\u{e9}" 6..8
Whitespace " " 8..9
Eof 9..9"#,
        ),
        (
            "\u{a0}x",
            r#"Character "\u{a0}x" 0..3
Eof 3..3"#,
        ),
    ]);
}

#[test]
fn tokenizes_preprocessor_errors_as_parse5() {
    check(&[
        (
            "a\u{1}b\u{7f}c\u{85}",
            r#"Character "a\u{1}b\u{7f}c\u{85}" 0..7
Eof 7..7
! control-character-in-input-stream 1..1
! control-character-in-input-stream 3..3
! control-character-in-input-stream 5..5"#,
        ),
        (
            "\u{b}\u{e}\u{1f}",
            r#"Character "\u{b}\u{e}\u{1f}" 0..3
Eof 3..3
! control-character-in-input-stream 0..0
! control-character-in-input-stream 1..1
! control-character-in-input-stream 2..2"#,
        ),
        (
            "\u{fdd0}\u{fdef}x\u{fffe}\u{ffff}",
            r#"Character "\u{fdd0}\u{fdef}x\u{fffe}\u{ffff}" 0..13
Eof 13..13
! noncharacter-in-input-stream 0..0
! noncharacter-in-input-stream 3..3
! noncharacter-in-input-stream 7..7
! noncharacter-in-input-stream 10..10"#,
        ),
        (
            "\u{1fffe}a\u{10ffff}",
            r#"Character "\u{1fffe}a\u{10ffff}" 0..9
Eof 9..9
! noncharacter-in-input-stream 0..0
! noncharacter-in-input-stream 5..5"#,
        ),
        (
            "<a\u{1} b=\"\u{2}\">\u{3}",
            r#"StartTag "a\u{1}" 0..10 "b"="\u{2}"@4..9
Character "\u{3}" 10..11
Eof 11..11
! control-character-in-input-stream 2..2
! control-character-in-input-stream 7..7
! control-character-in-input-stream 10..10"#,
        ),
        (
            "<!--\u{1}-->",
            r#"Comment "\u{1}" 0..8
Eof 8..8
! control-character-in-input-stream 4..4"#,
        ),
        (
            "&\u{1}",
            r#"Character "&\u{1}" 0..2
Eof 2..2
! control-character-in-input-stream 1..1"#,
        ),
        (
            "&#x\u{1}",
            r#"Character "&#x\u{1}" 0..4
Eof 4..4
! absence-of-digits-in-numeric-character-reference 3..3
! control-character-in-input-stream 3..3"#,
        ),
        (
            "\u{80}\u{9f}\u{a0}",
            r#"Character "\u{80}\u{9f}\u{a0}" 0..6
Eof 6..6
! control-character-in-input-stream 0..0
! control-character-in-input-stream 2..2"#,
        ),
        (
            "<title>\u{1}</title>",
            r#"StartTag "title" 0..7
Character "\u{1}" 7..8
EndTag "title" 8..16
Eof 16..16
! control-character-in-input-stream 7..7"#,
        ),
        (
            "&amp\u{1}",
            r#"Character "&\u{1}" 0..5
Eof 5..5
! missing-semicolon-after-character-reference 4..4
! control-character-in-input-stream 4..4"#,
        ),
    ]);
}

#[test]
fn tokenizes_tags_as_parse5() {
    check(&[
        (
            "<a>",
            r#"StartTag "a" 0..3
Eof 3..3"#,
        ),
        (
            "<A>",
            r#"StartTag "a" 0..3
Eof 3..3"#,
        ),
        (
            "<DIV class=X>",
            r#"StartTag "div" 0..13 "class"="X"@5..12
Eof 13..13"#,
        ),
        (
            "<a/>",
            r#"StartTag "a" 0..4 /
Eof 4..4"#,
        ),
        (
            "<br />",
            r#"StartTag "br" 0..6 /
Eof 6..6"#,
        ),
        (
            "<a  >",
            r#"StartTag "a" 0..5
Eof 5..5"#,
        ),
        (
            "</a>",
            r#"EndTag "a" 0..4
Eof 4..4"#,
        ),
        (
            "</A >",
            r#"EndTag "a" 0..5
Eof 5..5"#,
        ),
        (
            "<a\u{a}b>",
            r#"StartTag "a" 0..5 "b"=""@3..4
Eof 5..5"#,
        ),
        (
            "<a\u{9}b\u{c}c>",
            r#"StartTag "a" 0..7 "b"=""@3..4 "c"=""@5..6
Eof 7..7"#,
        ),
        (
            "<abc\u{0}d>",
            r#"StartTag "abc\u{fffd}d" 0..7
Eof 7..7
! unexpected-null-character 4..4"#,
        ),
        (
            "<a\u{e9}>",
            r#"StartTag "a\u{e9}" 0..5
Eof 5..5"#,
        ),
        (
            "<\u{e4}>",
            r#"Character "<\u{e4}>" 0..4
Eof 4..4
! invalid-first-character-of-tag-name 1..1"#,
        ),
        (
            "<1>",
            r#"Character "<1>" 0..3
Eof 3..3
! invalid-first-character-of-tag-name 1..1"#,
        ),
        (
            "< a>",
            r#"Character "<" 0..1
Whitespace " " 1..2
Character "a>" 2..4
Eof 4..4
! invalid-first-character-of-tag-name 1..1"#,
        ),
        (
            "<>",
            r#"Character "<>" 0..2
Eof 2..2
! invalid-first-character-of-tag-name 1..1"#,
        ),
        (
            "</>",
            r#"Eof 3..3
! missing-end-tag-name 2..2"#,
        ),
        (
            "</ >",
            r#"Comment " " 0..4
Eof 4..4
! invalid-first-character-of-tag-name 2..2"#,
        ),
        (
            "</1>",
            r#"Comment "1" 0..4
Eof 4..4
! invalid-first-character-of-tag-name 2..2"#,
        ),
        (
            "<?xml?>",
            r#"Comment "?xml?" 0..7
Eof 7..7
! unexpected-question-mark-instead-of-tag-name 1..1"#,
        ),
        (
            "<?\u{0}>",
            r#"Comment "?\u{fffd}" 0..4
Eof 4..4
! unexpected-question-mark-instead-of-tag-name 1..1
! unexpected-null-character 2..2"#,
        ),
        (
            "a<",
            r#"Character "a<" 0..2
Eof 2..2
! eof-before-tag-name 2..2"#,
        ),
        (
            "a</",
            r#"Character "a</" 0..3
Eof 3..3
! eof-before-tag-name 3..3"#,
        ),
        (
            "<a",
            r#"Eof 2..2
! eof-in-tag 2..2"#,
        ),
        (
            "</a",
            r#"Eof 3..3
! eof-in-tag 3..3"#,
        ),
        (
            "<a b",
            r#"Eof 4..4
! eof-in-tag 4..4"#,
        ),
        (
            "<a b=",
            r#"Eof 5..5
! eof-in-tag 5..5"#,
        ),
        (
            "<a b=\"c",
            r#"Eof 7..7
! eof-in-tag 7..7"#,
        ),
        (
            "<a b='c",
            r#"Eof 7..7
! eof-in-tag 7..7"#,
        ),
        (
            "<a b=c",
            r#"Eof 6..6
! eof-in-tag 6..6"#,
        ),
        (
            "<a b=\"c\"",
            r#"Eof 8..8
! eof-in-tag 8..8"#,
        ),
        (
            "<a /",
            r#"Eof 4..4
! eof-in-tag 4..4"#,
        ),
        (
            "</a b>",
            r#"EndTag "a" 0..6 "b"=""@4..5
Eof 6..6
! end-tag-with-attributes 5..5"#,
        ),
        (
            "</a/>",
            r#"EndTag "a" 0..5 /
Eof 5..5
! end-tag-with-trailing-solidus 4..4"#,
        ),
        (
            "<a/b>",
            r#"StartTag "a" 0..5 "b"=""@3..4
Eof 5..5
! unexpected-solidus-in-tag 3..3"#,
        ),
        (
            "<a / b>",
            r#"StartTag "a" 0..7 "b"=""@5..6
Eof 7..7
! unexpected-solidus-in-tag 4..4"#,
        ),
        (
            "<a//>",
            r#"StartTag "a" 0..5 /
Eof 5..5
! unexpected-solidus-in-tag 3..3"#,
        ),
        (
            "x<a>y</a>z",
            r#"Character "x" 0..1
StartTag "a" 1..4
Character "y" 4..5
EndTag "a" 5..9
Character "z" 9..10
Eof 10..10"#,
        ),
        (
            "<a>\u{a}",
            r#"StartTag "a" 0..3
Whitespace "\u{a}" 3..4
Eof 4..4"#,
        ),
        (
            "<a><b></b></a>",
            r#"StartTag "a" 0..3
StartTag "b" 3..6
EndTag "b" 6..10
EndTag "a" 10..14
Eof 14..14"#,
        ),
        (
            "<\u{1f600}>",
            r#"Character "<\u{1f600}>" 0..6
Eof 6..6
! invalid-first-character-of-tag-name 1..1"#,
        ),
        (
            "</\u{1f600}",
            r#"Comment "\u{1f600}" 0..7
Eof 6..6
! invalid-first-character-of-tag-name 2..2"#,
        ),
        (
            "<a\u{1f600}b>",
            r#"StartTag "a\u{1f600}b" 0..8
Eof 8..8"#,
        ),
    ]);
}

#[test]
fn tokenizes_attributes_as_parse5() {
    check(&[
        (
            "<a b c d>",
            r#"StartTag "a" 0..9 "b"=""@3..4 "c"=""@5..6 "d"=""@7..8
Eof 9..9"#,
        ),
        (
            "<a b=c d=e>",
            r#"StartTag "a" 0..11 "b"="c"@3..6 "d"="e"@7..10
Eof 11..11"#,
        ),
        (
            "<a b=\"c\" d='e'>",
            r#"StartTag "a" 0..15 "b"="c"@3..8 "d"="e"@9..14
Eof 15..15"#,
        ),
        (
            "<a b = \"c\" >",
            r#"StartTag "a" 0..12 "b"="c"@3..10
Eof 12..12"#,
        ),
        (
            "<a b=\"c\"d>",
            r#"StartTag "a" 0..10 "b"="c"@3..4 "d"=""@8..9
Eof 10..10
! missing-whitespace-between-attributes 8..8"#,
        ),
        (
            "<a b='c'd>",
            r#"StartTag "a" 0..10 "b"="c"@3..4 "d"=""@8..9
Eof 10..10
! missing-whitespace-between-attributes 8..8"#,
        ),
        (
            "<a b=\"c\"/>",
            r#"StartTag "a" 0..10 / "b"="c"@3..8
Eof 10..10"#,
        ),
        (
            "<a b=\"c\"/ d>",
            r#"StartTag "a" 0..12 "b"="c"@3..8 "d"=""@10..11
Eof 12..12
! unexpected-solidus-in-tag 9..9"#,
        ),
        (
            "<a B=1 b=2>",
            r#"StartTag "a" 0..11 "b"="1"@3..6
Eof 11..11
! duplicate-attribute 8..8"#,
        ),
        (
            "<a b=1 b=2 c=3 b=4>",
            r#"StartTag "a" 0..19 "b"="1"@3..6 "c"="3"@11..14
Eof 19..19
! duplicate-attribute 8..8
! duplicate-attribute 16..16"#,
        ),
        (
            "<a b=1 b=\"&amp;2\">",
            r#"StartTag "a" 0..18 "b"="1"@3..6
Eof 18..18
! duplicate-attribute 8..8"#,
        ),
        (
            "<a =b>",
            r#"StartTag "a" 0..6 "=b"=""@3..5
Eof 6..6
! unexpected-equals-sign-before-attribute-name 3..3"#,
        ),
        (
            "<a =\"b\">",
            r#"StartTag "a" 0..8 "=\u{22}b\u{22}"=""@3..7
Eof 8..8
! unexpected-equals-sign-before-attribute-name 3..3
! unexpected-character-in-attribute-name 4..4
! unexpected-character-in-attribute-name 6..6"#,
        ),
        (
            "<a b\"c'd<e=f>",
            r#"StartTag "a" 0..13 "b\u{22}c'd<e"="f"@3..12
Eof 13..13
! unexpected-character-in-attribute-name 4..4
! unexpected-character-in-attribute-name 6..6
! unexpected-character-in-attribute-name 8..8"#,
        ),
        (
            "<a b=c\"d'e<f=g`h>",
            r#"StartTag "a" 0..17 "b"="c\u{22}d'e<f=g`h"@3..16
Eof 17..17
! unexpected-character-in-unquoted-attribute-value 6..6
! unexpected-character-in-unquoted-attribute-value 8..8
! unexpected-character-in-unquoted-attribute-value 10..10
! unexpected-character-in-unquoted-attribute-value 12..12
! unexpected-character-in-unquoted-attribute-value 14..14"#,
        ),
        (
            "<a b=>",
            r#"StartTag "a" 0..6 "b"=""@3..4
Eof 6..6
! missing-attribute-value 5..5"#,
        ),
        (
            "<a b= >",
            r#"StartTag "a" 0..7 "b"=""@3..4
Eof 7..7
! missing-attribute-value 6..6"#,
        ),
        (
            "<a b\u{0}c=d\u{0}>",
            r#"StartTag "a" 0..10 "b\u{fffd}c"="d\u{fffd}"@3..9
Eof 10..10
! unexpected-null-character 4..4
! unexpected-null-character 8..8"#,
        ),
        (
            "<a b=\"\u{0}\" c='\u{0}'>",
            r#"StartTag "a" 0..15 "b"="\u{fffd}"@3..8 "c"="\u{fffd}"@9..14
Eof 15..15
! unexpected-null-character 6..6
! unexpected-null-character 12..12"#,
        ),
        (
            "<a b=\u{a}\"c\u{a}d\">",
            r#"StartTag "a" 0..12 "b"="c\u{a}d"@3..11
Eof 12..12"#,
        ),
        (
            "<a b=\"c&amp;d\" e='&lt;' f=&gt;>",
            r#"StartTag "a" 0..31 "b"="c&d"@3..14 "e"="<"@15..23 "f"=">"@24..30
Eof 31..31"#,
        ),
        (
            "<a b=\"&notin;&noti;&not\">",
            r#"StartTag "a" 0..25 "b"="\u{2209}&noti;\u{ac}"@3..24
Eof 25..25
! missing-semicolon-after-character-reference 23..23"#,
        ),
        (
            "<a b=\"&ampx\" c=\"&amp=\" d=\"&amp;x\" e=\"&amp\">",
            r#"StartTag "a" 0..43 "b"="&ampx"@3..12 "c"="&amp="@13..22 "d"="&x"@23..33 "e"="&"@34..42
Eof 43..43
! missing-semicolon-after-character-reference 41..41"#,
        ),
        (
            "<a b=&amp>",
            r#"StartTag "a" 0..10 "b"="&"@3..9
Eof 10..10
! missing-semicolon-after-character-reference 9..9"#,
        ),
        (
            "<a b=\"&#65\" c=&#x41>",
            r#"StartTag "a" 0..20 "b"="A"@3..11 "c"="A"@12..19
Eof 20..20
! missing-semicolon-after-character-reference 10..10
! missing-semicolon-after-character-reference 19..19"#,
        ),
        (
            "</a b=c>",
            r#"EndTag "a" 0..8 "b"="c"@4..7
Eof 8..8
! end-tag-with-attributes 7..7"#,
        ),
        (
            "</a b=c/>",
            r#"EndTag "a" 0..9 "b"="c/"@4..8
Eof 9..9
! end-tag-with-attributes 8..8"#,
        ),
        (
            "<a b=\"c\"'d'>",
            r#"StartTag "a" 0..12 "b"="c"@3..4 "'d'"=""@8..11
Eof 12..12
! missing-whitespace-between-attributes 8..8
! unexpected-character-in-attribute-name 8..8
! unexpected-character-in-attribute-name 10..10"#,
        ),
        (
            "<a \u{1f600}=\u{1f600}>",
            r#"StartTag "a" 0..13 "\u{1f600}"="\u{1f600}"@3..12
Eof 13..13"#,
        ),
        (
            "<a b=c/>",
            r#"StartTag "a" 0..8 "b"="c/"@3..7
Eof 8..8"#,
        ),
        (
            "<a b=\"c\u{d}\u{a}d\">",
            r#"StartTag "a" 0..12 "b"="c\u{a}d"@3..11
Eof 12..12"#,
        ),
        (
            "<a b c=d e>",
            r#"StartTag "a" 0..11 "b"=""@3..4 "c"="d"@5..8 "e"=""@9..10
Eof 11..11"#,
        ),
        (
            "<a\u{e9}=1 \u{c9}=2>",
            r#"StartTag "a\u{e9}=1" 0..12 "\u{c9}"="2"@7..11
Eof 12..12"#,
        ),
    ]);
}

#[test]
fn tokenizes_rcdata_rawtext_plaintext_as_parse5() {
    check(&[
        (
            "<title>a<b>&amp;</title>",
            r#"StartTag "title" 0..7
Character "a<b>&" 7..16
EndTag "title" 16..24
Eof 24..24"#,
        ),
        (
            "<title>a</titl></title >x",
            r#"StartTag "title" 0..7
Character "a</titl>" 7..15
EndTag "title" 15..24
Character "x" 24..25
Eof 25..25"#,
        ),
        (
            "<TITLE>a</TITLE>",
            r#"StartTag "title" 0..7
Character "a" 7..8
EndTag "title" 8..16
Eof 16..16"#,
        ),
        (
            "<title>a</title/>",
            r#"StartTag "title" 0..7
Character "a" 7..8
EndTag "title" 8..17 /
Eof 17..17
! end-tag-with-trailing-solidus 16..16"#,
        ),
        (
            "<title>a</title b=c>",
            r#"StartTag "title" 0..7
Character "a" 7..8
EndTag "title" 8..20 "b"="c"@16..19
Eof 20..20
! end-tag-with-attributes 19..19"#,
        ),
        (
            "<title>a</ti",
            r#"StartTag "title" 0..7
Character "a</ti" 7..12
Eof 12..12"#,
        ),
        (
            "<title>a</",
            r#"StartTag "title" 0..7
Character "a</" 7..10
Eof 10..10"#,
        ),
        (
            "<title>a<",
            r#"StartTag "title" 0..7
Character "a<" 7..9
Eof 9..9"#,
        ),
        (
            "<title>\u{0}</title>",
            r#"StartTag "title" 0..7
Character "\u{fffd}" 7..8
EndTag "title" 8..16
Eof 16..16
! unexpected-null-character 7..7"#,
        ),
        (
            "<title>&notit;</title>",
            r#"StartTag "title" 0..7
Character "\u{ac}it;" 7..14
EndTag "title" 14..22
Eof 22..22
! missing-semicolon-after-character-reference 11..11"#,
        ),
        (
            "<textarea>a</textarea>b",
            r#"StartTag "textarea" 0..10
Character "a" 10..11
EndTag "textarea" 11..22
Character "b" 22..23
Eof 23..23"#,
        ),
        (
            "<style>a<b>&amp;</style>",
            r#"StartTag "style" 0..7
Character "a<b>&amp;" 7..16
EndTag "style" 16..24
Eof 24..24"#,
        ),
        (
            "<style>a</styles></style>",
            r#"StartTag "style" 0..7
Character "a</styles>" 7..17
EndTag "style" 17..25
Eof 25..25"#,
        ),
        (
            "<style>a</",
            r#"StartTag "style" 0..7
Character "a</" 7..10
Eof 10..10"#,
        ),
        (
            "<style>\u{0}",
            r#"StartTag "style" 0..7
Character "\u{fffd}" 7..8
Eof 8..8
! unexpected-null-character 7..7"#,
        ),
        (
            "<xmp></x></xmp>",
            r#"StartTag "xmp" 0..5
Character "</x>" 5..9
EndTag "xmp" 9..15
Eof 15..15"#,
        ),
        (
            "<iframe>a</iframe\u{a}>",
            r#"StartTag "iframe" 0..8
Character "a" 8..9
EndTag "iframe" 9..19
Eof 19..19"#,
        ),
        (
            "<noembed>a</noembed>",
            r#"StartTag "noembed" 0..9
Character "a" 9..10
EndTag "noembed" 10..20
Eof 20..20"#,
        ),
        (
            "<noframes>a</noframes>",
            r#"StartTag "noframes" 0..10
Character "a" 10..11
EndTag "noframes" 11..22
Eof 22..22"#,
        ),
        (
            "<plaintext>a</plaintext><b>&amp;\u{0}",
            r#"StartTag "plaintext" 0..11
Character "a</plaintext><b>&amp;\u{fffd}" 11..33
Eof 33..33
! unexpected-null-character 32..32"#,
        ),
        (
            "<title>a</title",
            r#"StartTag "title" 0..7
Character "a</title" 7..15
Eof 15..15"#,
        ),
        (
            "<title>a</title ",
            r#"StartTag "title" 0..7
Character "a" 7..16
Eof 16..16
! eof-in-tag 16..16"#,
        ),
        (
            "<title></title\u{9}x=\"y\">z",
            r#"StartTag "title" 0..7
EndTag "title" 7..21 "x"="y"@15..20
Character "z" 21..22
Eof 22..22
! end-tag-with-attributes 20..20"#,
        ),
        (
            "<title>a</title1>",
            r#"StartTag "title" 0..7
Character "a</title1>" 7..17
Eof 17..17"#,
        ),
        (
            "<title>a</TiTlE>",
            r#"StartTag "title" 0..7
Character "a" 7..8
EndTag "title" 8..16
Eof 16..16"#,
        ),
        (
            "<style>&amp;</style>",
            r#"StartTag "style" 0..7
Character "&amp;" 7..12
EndTag "style" 12..20
Eof 20..20"#,
        ),
        (
            "<title>a\u{d}\u{a}</title>",
            r#"StartTag "title" 0..7
Character "a" 7..8
Whitespace "\u{a}" 8..10
EndTag "title" 10..18
Eof 18..18"#,
        ),
    ]);
}

#[test]
fn tokenizes_script_data_as_parse5() {
    check(&[
        (
            "<script>a<b></script>",
            r#"StartTag "script" 0..8
Character "a<b>" 8..12
EndTag "script" 12..21
Eof 21..21"#,
        ),
        (
            "<script>a</scrip></script>",
            r#"StartTag "script" 0..8
Character "a</scrip>" 8..17
EndTag "script" 17..26
Eof 26..26"#,
        ),
        (
            "<script></script >",
            r#"StartTag "script" 0..8
EndTag "script" 8..18
Eof 18..18"#,
        ),
        (
            "<script>\u{0}</script>",
            r#"StartTag "script" 0..8
Character "\u{fffd}" 8..9
EndTag "script" 9..18
Eof 18..18
! unexpected-null-character 8..8"#,
        ),
        (
            "<script><!--a--></script>",
            r#"StartTag "script" 0..8
Character "<!--a-->" 8..16
EndTag "script" 16..25
Eof 25..25"#,
        ),
        (
            "<script><!-a</script>",
            r#"StartTag "script" 0..8
Character "<!-a" 8..12
EndTag "script" 12..21
Eof 21..21"#,
        ),
        (
            "<script><!</script>",
            r#"StartTag "script" 0..8
Character "<!" 8..10
EndTag "script" 10..19
Eof 19..19"#,
        ),
        (
            "<script><!--</script>",
            r#"StartTag "script" 0..8
Character "<!--" 8..12
EndTag "script" 12..21
Eof 21..21"#,
        ),
        (
            "<script><!-- a - b -- c --></script>",
            r#"StartTag "script" 0..8
Character "<!--" 8..12
Whitespace " " 12..13
Character "a" 13..14
Whitespace " " 14..15
Character "-" 15..16
Whitespace " " 16..17
Character "b" 17..18
Whitespace " " 18..19
Character "--" 19..21
Whitespace " " 21..22
Character "c" 22..23
Whitespace " " 23..24
Character "-->" 24..27
EndTag "script" 27..36
Eof 36..36"#,
        ),
        (
            "<script><!--<script>a</script>b</script>c",
            r#"StartTag "script" 0..8
Character "<!--<script>a</script>b" 8..31
EndTag "script" 31..40
Character "c" 40..41
Eof 41..41"#,
        ),
        (
            "<script><!--<script>-a--b<c</script>--></script>",
            r#"StartTag "script" 0..8
Character "<!--<script>-a--b<c</script>-->" 8..39
EndTag "script" 39..48
Eof 48..48"#,
        ),
        (
            "<script><!--<SCRIPT>a</SCRIPT>--></script>",
            r#"StartTag "script" 0..8
Character "<!--<SCRIPT>a</SCRIPT>-->" 8..33
EndTag "script" 33..42
Eof 42..42"#,
        ),
        (
            "<script><!--<scripts>a</script>",
            r#"StartTag "script" 0..8
Character "<!--<scripts>a" 8..22
EndTag "script" 22..31
Eof 31..31"#,
        ),
        (
            "<script><!--<script\u{a}>-\u{0}--\u{0}<\u{0}</script>",
            r#"StartTag "script" 0..8
Character "<!--<script" 8..19
Whitespace "\u{a}" 19..20
Character ">-\u{fffd}--\u{fffd}<\u{fffd}</script>" 20..37
Eof 37..37
! unexpected-null-character 22..22
! unexpected-null-character 25..25
! unexpected-null-character 27..27
! eof-in-script-html-comment-like-text 37..37"#,
        ),
        (
            "<script><!--<script/></script/>--></script>",
            r#"StartTag "script" 0..8
Character "<!--<script/></script/>-->" 8..34
EndTag "script" 34..43
Eof 43..43"#,
        ),
        (
            "<script><!--<script>",
            r#"StartTag "script" 0..8
Character "<!--<script>" 8..20
Eof 20..20
! eof-in-script-html-comment-like-text 20..20"#,
        ),
        (
            "<script><!--<script>-",
            r#"StartTag "script" 0..8
Character "<!--<script>-" 8..21
Eof 21..21
! eof-in-script-html-comment-like-text 21..21"#,
        ),
        (
            "<script><!--<script>--",
            r#"StartTag "script" 0..8
Character "<!--<script>--" 8..22
Eof 22..22
! eof-in-script-html-comment-like-text 22..22"#,
        ),
        (
            "<script><!--<script><",
            r#"StartTag "script" 0..8
Character "<!--<script><" 8..21
Eof 21..21
! eof-in-script-html-comment-like-text 21..21"#,
        ),
        (
            "<script><!--<script></",
            r#"StartTag "script" 0..8
Character "<!--<script></" 8..22
Eof 22..22
! eof-in-script-html-comment-like-text 22..22"#,
        ),
        (
            "<script><!--<script></script",
            r#"StartTag "script" 0..8
Character "<!--<script></script" 8..28
Eof 28..28
! eof-in-script-html-comment-like-text 28..28"#,
        ),
        (
            "<script><!--",
            r#"StartTag "script" 0..8
Character "<!--" 8..12
Eof 12..12
! eof-in-script-html-comment-like-text 12..12"#,
        ),
        (
            "<script><!---",
            r#"StartTag "script" 0..8
Character "<!---" 8..13
Eof 13..13
! eof-in-script-html-comment-like-text 13..13"#,
        ),
        (
            "<script><!--a-",
            r#"StartTag "script" 0..8
Character "<!--a-" 8..14
Eof 14..14
! eof-in-script-html-comment-like-text 14..14"#,
        ),
        (
            "<script><!--a--",
            r#"StartTag "script" 0..8
Character "<!--a--" 8..15
Eof 15..15
! eof-in-script-html-comment-like-text 15..15"#,
        ),
        (
            "<script><!--a<",
            r#"StartTag "script" 0..8
Character "<!--a<" 8..14
Eof 14..14
! eof-in-script-html-comment-like-text 14..14"#,
        ),
        (
            "<script><!--a</",
            r#"StartTag "script" 0..8
Character "<!--a</" 8..15
Eof 15..15
! eof-in-script-html-comment-like-text 15..15"#,
        ),
        (
            "<script><!--a</s",
            r#"StartTag "script" 0..8
Character "<!--a</s" 8..16
Eof 16..16
! eof-in-script-html-comment-like-text 16..16"#,
        ),
        (
            "<script><!--a<b",
            r#"StartTag "script" 0..8
Character "<!--a<b" 8..15
Eof 15..15
! eof-in-script-html-comment-like-text 15..15"#,
        ),
        (
            "<script><!--\u{0}-\u{0}",
            r#"StartTag "script" 0..8
Character "<!--\u{fffd}-\u{fffd}" 8..15
Eof 15..15
! unexpected-null-character 12..12
! unexpected-null-character 14..14
! eof-in-script-html-comment-like-text 15..15"#,
        ),
        (
            "<script><!--a<1</x--></script>",
            r#"StartTag "script" 0..8
Character "<!--a<1</x-->" 8..21
EndTag "script" 21..30
Eof 30..30"#,
        ),
        (
            "<script><!--a</script >",
            r#"StartTag "script" 0..8
Character "<!--a" 8..13
EndTag "script" 13..23
Eof 23..23"#,
        ),
        (
            "<script>a</script",
            r#"StartTag "script" 0..8
Character "a</script" 8..17
Eof 17..17"#,
        ),
        (
            "<script><!--<script>\u{0}-<>",
            r#"StartTag "script" 0..8
Character "<!--<script>\u{fffd}-<>" 8..24
Eof 24..24
! unexpected-null-character 20..20
! eof-in-script-html-comment-like-text 24..24"#,
        ),
        (
            "<script><!--<script>-<x",
            r#"StartTag "script" 0..8
Character "<!--<script>-<x" 8..23
Eof 23..23
! eof-in-script-html-comment-like-text 23..23"#,
        ),
        (
            "<script><!--<script>--<x",
            r#"StartTag "script" 0..8
Character "<!--<script>--<x" 8..24
Eof 24..24
! eof-in-script-html-comment-like-text 24..24"#,
        ),
        (
            "<script><!--a->--></script>",
            r#"StartTag "script" 0..8
Character "<!--a->-->" 8..18
EndTag "script" 18..27
Eof 27..27"#,
        ),
        (
            "<script>a<!-->b</script>",
            r#"StartTag "script" 0..8
Character "a<!-->b" 8..15
EndTag "script" 15..24
Eof 24..24"#,
        ),
    ]);
}

#[test]
fn tokenizes_comments_as_parse5() {
    check(&[
        (
            "<!---->",
            r#"Comment "" 0..7
Eof 7..7"#,
        ),
        (
            "<!-- a -->",
            r#"Comment " a " 0..10
Eof 10..10"#,
        ),
        (
            "<!-->",
            r#"Comment "" 0..5
Eof 5..5
! abrupt-closing-of-empty-comment 4..4"#,
        ),
        (
            "<!--->",
            r#"Comment "" 0..6
Eof 6..6
! abrupt-closing-of-empty-comment 5..5"#,
        ),
        (
            "<!--a",
            r#"Comment "a" 0..6
Eof 5..5
! eof-in-comment 5..5"#,
        ),
        (
            "<!--a-",
            r#"Comment "a" 0..7
Eof 6..6
! eof-in-comment 6..6"#,
        ),
        (
            "<!--a--",
            r#"Comment "a" 0..8
Eof 7..7
! eof-in-comment 7..7"#,
        ),
        (
            "<!--a--!",
            r#"Comment "a" 0..9
Eof 8..8
! eof-in-comment 8..8"#,
        ),
        (
            "<!--a--!>",
            r#"Comment "a" 0..9
Eof 9..9
! incorrectly-closed-comment 8..8"#,
        ),
        (
            "<!--a--!b-->",
            r#"Comment "a--!b" 0..12
Eof 12..12"#,
        ),
        (
            "<!--a--!-->",
            r#"Comment "a--!" 0..11
Eof 11..11"#,
        ),
        (
            "<!--a---->",
            r#"Comment "a--" 0..10
Eof 10..10"#,
        ),
        (
            "<!--a-b-->",
            r#"Comment "a-b" 0..10
Eof 10..10"#,
        ),
        (
            "<!--<!---->",
            r#"Comment "<!--" 0..11
Eof 11..11
! nested-comment 8..8"#,
        ),
        (
            "<!--<!--a-->",
            r#"Comment "<!--a" 0..12
Eof 12..12
! nested-comment 8..8"#,
        ),
        (
            "<!--<!-->",
            r#"Comment "<!" 0..9
Eof 9..9"#,
        ),
        (
            "<!--<!-",
            r#"Comment "<!" 0..8
Eof 7..7
! eof-in-comment 7..7"#,
        ),
        (
            "<!--<!--",
            r#"Comment "<!" 0..9
Eof 8..8
! eof-in-comment 8..8"#,
        ),
        (
            "<!--<!x-->",
            r#"Comment "<!x" 0..10
Eof 10..10"#,
        ),
        (
            "<!--<<!--->",
            r#"Comment "<<!-" 0..11
Eof 11..11
! nested-comment 9..9"#,
        ),
        (
            "<!--a\u{0}b-->",
            r#"Comment "a\u{fffd}b" 0..10
Eof 10..10
! unexpected-null-character 5..5"#,
        ),
        (
            "<!--",
            r#"Comment "" 0..5
Eof 4..4
! eof-in-comment 4..4"#,
        ),
        (
            "<!--\u{0}",
            r#"Comment "\u{fffd}" 0..6
Eof 5..5
! unexpected-null-character 4..4
! eof-in-comment 5..5"#,
        ),
        (
            "<!x>",
            r#"Comment "x" 0..4
Eof 4..4
! incorrectly-opened-comment 2..2"#,
        ),
        (
            "<!>",
            r#"Comment "" 0..3
Eof 3..3
! incorrectly-opened-comment 2..2"#,
        ),
        (
            "<!",
            r#"Comment "" 0..3
Eof 2..2
! incorrectly-opened-comment 2..2"#,
        ),
        (
            "<!-",
            r#"Comment "-" 0..4
Eof 3..3
! incorrectly-opened-comment 2..2"#,
        ),
        (
            "<!-x>",
            r#"Comment "-x" 0..5
Eof 5..5
! incorrectly-opened-comment 2..2"#,
        ),
        (
            "<!DOCTYP>",
            r#"Comment "DOCTYP" 0..9
Eof 9..9
! incorrectly-opened-comment 2..2"#,
        ),
        (
            "<![CDATA[a]]>",
            r#"Comment "[CDATA[a]]" 0..13
Eof 13..13
! cdata-in-html-content 8..8"#,
        ),
        (
            "<![cdata[a]]>",
            r#"Comment "[cdata[a]]" 0..13
Eof 13..13
! incorrectly-opened-comment 2..2"#,
        ),
        (
            "<!-- a -- b -->",
            r#"Comment " a -- b " 0..15
Eof 15..15"#,
        ),
        (
            "<!--a--\u{a}>-->",
            r#"Comment "a--\u{a}>" 0..12
Eof 12..12"#,
        ),
        (
            "<!--a<b-->",
            r#"Comment "a<b" 0..10
Eof 10..10"#,
        ),
        (
            "<!--a<!b-->",
            r#"Comment "a<!b" 0..11
Eof 11..11"#,
        ),
        (
            "<!--a<!-b-->",
            r#"Comment "a<!-b" 0..12
Eof 12..12"#,
        ),
        (
            "<!--a\u{d}\u{a}b-->",
            r#"Comment "a\u{a}b" 0..11
Eof 11..11"#,
        ),
        (
            "x<!--y-->z",
            r#"Character "x" 0..1
Comment "y" 1..9
Character "z" 9..10
Eof 10..10"#,
        ),
        (
            "<!--\u{1f600}-->",
            r#"Comment "\u{1f600}" 0..11
Eof 11..11"#,
        ),
        (
            "<!\u{1f600}>",
            r#"Comment "\u{1f600}" 0..7
Eof 7..7
! incorrectly-opened-comment 2..2"#,
        ),
        (
            "<?\u{1f600}",
            r#"Comment "?\u{1f600}" 0..7
Eof 6..6
! unexpected-question-mark-instead-of-tag-name 1..1"#,
        ),
        (
            "<!--a--!\u{0}-->",
            r#"Comment "a--!\u{fffd}" 0..12
Eof 12..12
! unexpected-null-character 8..8"#,
        ),
        (
            "<!--<!--->",
            r#"Comment "<!-" 0..10
Eof 10..10
! nested-comment 8..8"#,
        ),
    ]);
}

#[test]
fn tokenizes_doctypes_as_parse5() {
    check(&[
        (
            "<!DOCTYPE html>",
            r#"Doctype "html" null null 0..15
Eof 15..15"#,
        ),
        (
            "<!doctype HTML>",
            r#"Doctype "html" null null 0..15
Eof 15..15"#,
        ),
        (
            "<!DOCTYPE>",
            r#"Doctype null null null 0..10 quirks
Eof 10..10
! missing-doctype-name 9..9"#,
        ),
        (
            "<!DOCTYPE >",
            r#"Doctype null null null 0..11 quirks
Eof 11..11
! missing-doctype-name 10..10"#,
        ),
        (
            "<!DOCTYPEhtml>",
            r#"Doctype "html" null null 0..14
Eof 14..14
! missing-whitespace-before-doctype-name 9..9"#,
        ),
        (
            "<!DOCTYPE",
            r#"Doctype null null null 0..10 quirks
Eof 9..9
! eof-in-doctype 9..9"#,
        ),
        (
            "<!DOCTYPE ",
            r#"Doctype null null null 0..11 quirks
Eof 10..10
! eof-in-doctype 10..10"#,
        ),
        (
            "<!DOCTYPE h",
            r#"Doctype "h" null null 0..12 quirks
Eof 11..11
! eof-in-doctype 11..11"#,
        ),
        (
            "<!DOCTYPE html ",
            r#"Doctype "html" null null 0..16 quirks
Eof 15..15
! eof-in-doctype 15..15"#,
        ),
        (
            "<!DOCTYPE \u{0}x>",
            r#"Doctype "\u{fffd}x" null null 0..13
Eof 13..13
! unexpected-null-character 10..10"#,
        ),
        (
            "<!DOCTYPE h\u{0}>",
            r#"Doctype "h\u{fffd}" null null 0..13
Eof 13..13
! unexpected-null-character 11..11"#,
        ),
        (
            "<!DOCTYPE html PUBLIC \"-//W3C//DTD HTML 4.01//EN\" \"http://www.w3.org/TR/html4/strict.dtd\">",
            r#"Doctype "html" "-//W3C//DTD HTML 4.01//EN" "http://www.w3.org/TR/html4/strict.dtd" 0..90
Eof 90..90"#,
        ),
        (
            "<!DOCTYPE html PUBLIC '-//a' 'b'>",
            r#"Doctype "html" "-//a" "b" 0..33
Eof 33..33"#,
        ),
        (
            "<!DOCTYPE html public\"a\"system\"b\">",
            r#"Doctype "html" "a" null 0..34 quirks
Eof 34..34
! missing-whitespace-after-doctype-public-keyword 21..21
! missing-quote-before-doctype-system-identifier 24..24"#,
        ),
        (
            "<!DOCTYPE html PUBLIC \"a\"'b'>",
            r#"Doctype "html" "a" "b" 0..29
Eof 29..29
! missing-whitespace-between-doctype-public-and-system-identifiers 25..25"#,
        ),
        (
            "<!DOCTYPE html PUBLIC \"a\">",
            r#"Doctype "html" "a" null 0..26
Eof 26..26"#,
        ),
        (
            "<!DOCTYPE html PUBLIC \"a\" >",
            r#"Doctype "html" "a" null 0..27
Eof 27..27"#,
        ),
        (
            "<!DOCTYPE html PUBLIC>",
            r#"Doctype "html" null null 0..22 quirks
Eof 22..22
! missing-doctype-public-identifier 21..21"#,
        ),
        (
            "<!DOCTYPE html PUBLIC >",
            r#"Doctype "html" null null 0..23 quirks
Eof 23..23
! missing-doctype-public-identifier 22..22"#,
        ),
        (
            "<!DOCTYPE html PUBLIC x>",
            r#"Doctype "html" null null 0..24 quirks
Eof 24..24
! missing-quote-before-doctype-public-identifier 22..22"#,
        ),
        (
            "<!DOCTYPE html PUBLICx>",
            r#"Doctype "html" null null 0..23 quirks
Eof 23..23
! missing-quote-before-doctype-public-identifier 21..21"#,
        ),
        (
            "<!DOCTYPE html PUBLIC \"a>",
            r#"Doctype "html" "a" null 0..25 quirks
Eof 25..25
! abrupt-doctype-public-identifier 24..24"#,
        ),
        (
            "<!DOCTYPE html PUBLIC 'a>",
            r#"Doctype "html" "a" null 0..25 quirks
Eof 25..25
! abrupt-doctype-public-identifier 24..24"#,
        ),
        (
            "<!DOCTYPE html PUBLIC \"a",
            r#"Doctype "html" "a" null 0..25 quirks
Eof 24..24
! eof-in-doctype 24..24"#,
        ),
        (
            "<!DOCTYPE html PUBLIC \"a\" \"b",
            r#"Doctype "html" "a" "b" 0..29 quirks
Eof 28..28
! eof-in-doctype 28..28"#,
        ),
        (
            "<!DOCTYPE html PUBLIC \"a\" x>",
            r#"Doctype "html" "a" null 0..28 quirks
Eof 28..28
! missing-quote-before-doctype-system-identifier 26..26"#,
        ),
        (
            "<!DOCTYPE html PUBLIC \"a\"x>",
            r#"Doctype "html" "a" null 0..27 quirks
Eof 27..27
! missing-quote-before-doctype-system-identifier 25..25"#,
        ),
        (
            "<!DOCTYPE html SYSTEM \"a\">",
            r#"Doctype "html" null "a" 0..26
Eof 26..26"#,
        ),
        (
            "<!DOCTYPE html SYSTEM 'a' >",
            r#"Doctype "html" null "a" 0..27
Eof 27..27"#,
        ),
        (
            "<!DOCTYPE html SYSTEM\"a\">",
            r#"Doctype "html" null "a" 0..25
Eof 25..25
! missing-whitespace-after-doctype-system-keyword 21..21"#,
        ),
        (
            "<!DOCTYPE html SYSTEM'a'>",
            r#"Doctype "html" null "a" 0..25
Eof 25..25
! missing-whitespace-after-doctype-system-keyword 21..21"#,
        ),
        (
            "<!DOCTYPE html SYSTEM>",
            r#"Doctype "html" null null 0..22 quirks
Eof 22..22
! missing-doctype-system-identifier 21..21"#,
        ),
        (
            "<!DOCTYPE html SYSTEM >",
            r#"Doctype "html" null null 0..23 quirks
Eof 23..23
! missing-doctype-system-identifier 22..22"#,
        ),
        (
            "<!DOCTYPE html SYSTEM x>",
            r#"Doctype "html" null null 0..24 quirks
Eof 24..24
! missing-quote-before-doctype-system-identifier 22..22"#,
        ),
        (
            "<!DOCTYPE html SYSTEM \"a>",
            r#"Doctype "html" null "a" 0..25 quirks
Eof 25..25
! abrupt-doctype-system-identifier 24..24"#,
        ),
        (
            "<!DOCTYPE html SYSTEM \"a\" x>",
            r#"Doctype "html" null "a" 0..28
Eof 28..28
! unexpected-character-after-doctype-system-identifier 26..26"#,
        ),
        (
            "<!DOCTYPE html SYSTEM \"a",
            r#"Doctype "html" null "a" 0..25 quirks
Eof 24..24
! eof-in-doctype 24..24"#,
        ),
        (
            "<!DOCTYPE html SYSTEM",
            r#"Doctype "html" null null 0..22 quirks
Eof 21..21
! eof-in-doctype 21..21"#,
        ),
        (
            "<!DOCTYPE html PUBLIC",
            r#"Doctype "html" null null 0..22 quirks
Eof 21..21
! eof-in-doctype 21..21"#,
        ),
        (
            "<!DOCTYPE html PUBLIC \"\u{0}\"  \"\u{0}\">",
            r#"Doctype "html" "\u{fffd}" "\u{fffd}" 0..31
Eof 31..31
! unexpected-null-character 23..23
! unexpected-null-character 28..28"#,
        ),
        (
            "<!DOCTYPE html PUBLIC '\u{0}' '\u{0}'>",
            r#"Doctype "html" "\u{fffd}" "\u{fffd}" 0..30
Eof 30..30
! unexpected-null-character 23..23
! unexpected-null-character 27..27"#,
        ),
        (
            "<!DOCTYPE html foo>",
            r#"Doctype "html" null null 0..19 quirks
Eof 19..19
! invalid-character-sequence-after-doctype-name 15..15"#,
        ),
        (
            "<!DOCTYPE html foo\u{0} bar",
            r#"Doctype "html" null null 0..24 quirks
Eof 23..23
! invalid-character-sequence-after-doctype-name 15..15
! unexpected-null-character 18..18"#,
        ),
        (
            "<!DOCTYPE html SYSTEM \"a\" b\u{0}>",
            r#"Doctype "html" null "a" 0..29
Eof 29..29
! unexpected-character-after-doctype-system-identifier 26..26
! unexpected-null-character 27..27"#,
        ),
        (
            "<!DOCTYPE html PUBL",
            r#"Doctype "html" null null 0..20 quirks
Eof 19..19
! invalid-character-sequence-after-doctype-name 15..15"#,
        ),
        (
            "<!DOCTYPE html SYST>",
            r#"Doctype "html" null null 0..20 quirks
Eof 20..20
! invalid-character-sequence-after-doctype-name 15..15"#,
        ),
        (
            "x<!DOCTYPE html>",
            r#"Character "x" 0..1
Doctype "html" null null 1..16
Eof 16..16"#,
        ),
        (
            " <!DOCTYPE html>",
            r#"Whitespace " " 0..1
Doctype "html" null null 1..16
Eof 16..16"#,
        ),
        (
            "<!DOCTYPE html\u{d}\u{a}PUBLIC\u{d}\u{a}\"a\"\u{d}\u{a}\"b\"\u{d}\u{a}>",
            r#"Doctype "html" "a" "b" 0..35
Eof 35..35"#,
        ),
        (
            "<!DOCTYPE HTML PUBLIC \"A\" SYSTEM \"B\">",
            r#"Doctype "html" "A" null 0..37 quirks
Eof 37..37
! missing-quote-before-doctype-system-identifier 26..26"#,
        ),
        (
            "<!DOCTYPE html PUBLIC \"a\" 'b",
            r#"Doctype "html" "a" "b" 0..29 quirks
Eof 28..28
! eof-in-doctype 28..28"#,
        ),
        (
            "<!DOCTYPE html PUBLIC \"a\"\u{a}\u{a}",
            r#"Doctype "html" "a" null 0..28 quirks
Eof 27..27
! eof-in-doctype 27..27"#,
        ),
        (
            "<!DOCTYPE html SYSTEM \"a\"\u{a}",
            r#"Doctype "html" null "a" 0..27 quirks
Eof 26..26
! eof-in-doctype 26..26"#,
        ),
        (
            "<!DOCTYPE \u{1f600}>",
            r#"Doctype "\u{1f600}" null null 0..15
Eof 15..15"#,
        ),
        (
            "<!DOCTYPE html PUBLIC \"a\" \"b\">x",
            r#"Doctype "html" "a" "b" 0..30
Character "x" 30..31
Eof 31..31"#,
        ),
        (
            "<!DOCTYPE html PUBLIC  \"a\"  \"b\"  >",
            r#"Doctype "html" "a" "b" 0..34
Eof 34..34"#,
        ),
    ]);
}

#[test]
fn tokenizes_cdata_as_parse5() {
    check(&[
        (
            "<svg><![CDATA[a<b>&amp;]]></svg>",
            r#"StartTag "svg" 0..5
Character "a<b>&amp;" 5..26
EndTag "svg" 26..32
Eof 32..32"#,
        ),
        (
            "<svg><![CDATA[a]b]]c]]]>x",
            r#"StartTag "svg" 0..5
Character "a]b]]c]x" 5..25
Eof 25..25"#,
        ),
        (
            "<svg><![CDATA[a",
            r#"StartTag "svg" 0..5
Character "a" 5..15
Eof 15..15
! eof-in-cdata 15..15"#,
        ),
        (
            "<svg><![CDATA[a]",
            r#"StartTag "svg" 0..5
Character "a]" 5..16
Eof 16..16
! eof-in-cdata 16..16"#,
        ),
        (
            "<svg><![CDATA[a]]",
            r#"StartTag "svg" 0..5
Character "a]]" 5..17
Eof 17..17
! eof-in-cdata 17..17"#,
        ),
        (
            "<svg><![CDATA[]]></svg>",
            r#"StartTag "svg" 0..5
EndTag "svg" 17..23
Eof 23..23"#,
        ),
        (
            "<math><![CDATA[ \u{0} ]]>",
            r#"StartTag "math" 0..6
Whitespace " " 6..16
Null "\u{0}" 16..17
Whitespace " " 17..21
Eof 21..21"#,
        ),
        (
            "<svg></svg><![CDATA[a]]>",
            r#"StartTag "svg" 0..5
EndTag "svg" 5..11
Comment "[CDATA[a]]" 11..24
Eof 24..24
! cdata-in-html-content 19..19"#,
        ),
        (
            "<svg><![CDATA[a\u{d}\u{a}b]]>",
            r#"StartTag "svg" 0..5
Character "a" 5..15
Whitespace "\u{a}" 15..17
Character "b" 17..21
Eof 21..21"#,
        ),
        (
            "<svg><![CDATA",
            r#"StartTag "svg" 0..5
Comment "[CDATA" 5..14
Eof 13..13
! incorrectly-opened-comment 7..7"#,
        ),
        (
            "<svg><![cdata[a]]>",
            r#"StartTag "svg" 0..5
Comment "[cdata[a]]" 5..18
Eof 18..18
! incorrectly-opened-comment 7..7"#,
        ),
        (
            "a<svg><![CDATA[b]]>c",
            r#"Character "a" 0..1
StartTag "svg" 1..6
Character "bc" 6..20
Eof 20..20"#,
        ),
        (
            "<svg><![CDATA[]]]]]>",
            r#"StartTag "svg" 0..5
Character "]]]" 5..20
Eof 20..20"#,
        ),
    ]);
}

#[test]
fn tokenizes_references_as_parse5() {
    check(&[
        (
            "&amp;",
            r#"Character "&" 0..5
Eof 5..5"#,
        ),
        (
            "&amp",
            r#"Character "&" 0..4
Eof 4..4
! missing-semicolon-after-character-reference 4..4"#,
        ),
        (
            "&AMP;",
            r#"Character "&" 0..5
Eof 5..5"#,
        ),
        (
            "&ampx",
            r#"Character "&x" 0..5
Eof 5..5
! missing-semicolon-after-character-reference 4..4"#,
        ),
        (
            "&amp;x",
            r#"Character "&x" 0..6
Eof 6..6"#,
        ),
        (
            "&notit;",
            r#"Character "\u{ac}it;" 0..7
Eof 7..7
! missing-semicolon-after-character-reference 4..4"#,
        ),
        (
            "&notin;",
            r#"Character "\u{2209}" 0..7
Eof 7..7"#,
        ),
        (
            "&noti",
            r#"Character "\u{ac}i" 0..5
Eof 5..5
! missing-semicolon-after-character-reference 4..4"#,
        ),
        (
            "&not",
            r#"Character "\u{ac}" 0..4
Eof 4..4
! missing-semicolon-after-character-reference 4..4"#,
        ),
        (
            "&no",
            r#"Character "&no" 0..3
Eof 3..3"#,
        ),
        (
            "&n",
            r#"Character "&n" 0..2
Eof 2..2"#,
        ),
        (
            "&",
            r#"Character "&" 0..1
Eof 1..1"#,
        ),
        (
            "&;",
            r#"Character "&;" 0..2
Eof 2..2"#,
        ),
        (
            "& ",
            r#"Character "&" 0..1
Whitespace " " 1..2
Eof 2..2"#,
        ),
        (
            "&x;",
            r#"Character "&x;" 0..3
Eof 3..3
! unknown-named-character-reference 2..2"#,
        ),
        (
            "&xyz;",
            r#"Character "&xyz;" 0..5
Eof 5..5
! unknown-named-character-reference 4..4"#,
        ),
        (
            "&xyz",
            r#"Character "&xyz" 0..4
Eof 4..4"#,
        ),
        (
            "&abc1;",
            r#"Character "&abc1;" 0..6
Eof 6..6
! unknown-named-character-reference 5..5"#,
        ),
        (
            "&1;",
            r#"Character "&1;" 0..3
Eof 3..3
! unknown-named-character-reference 2..2"#,
        ),
        (
            "&;x",
            r#"Character "&;x" 0..3
Eof 3..3"#,
        ),
        (
            "&&amp;",
            r#"Character "&&" 0..6
Eof 6..6"#,
        ),
        (
            "&a&b",
            r#"Character "&a&b" 0..4
Eof 4..4"#,
        ),
        (
            "&lt;&gt;&quot;&apos;",
            r#"Character "<>\u{22}'" 0..20
Eof 20..20"#,
        ),
        (
            "&NotEqualTilde;",
            r#"Character "\u{2242}\u{338}" 0..15
Eof 15..15"#,
        ),
        (
            "&nGt;",
            r#"Character "\u{226b}\u{20d2}" 0..5
Eof 5..5"#,
        ),
        (
            "&acE;",
            r#"Character "\u{223e}\u{333}" 0..5
Eof 5..5"#,
        ),
        (
            "&Aacute",
            r#"Character "\u{c1}" 0..7
Eof 7..7
! missing-semicolon-after-character-reference 7..7"#,
        ),
        (
            "&Aacutex",
            r#"Character "\u{c1}x" 0..8
Eof 8..8
! missing-semicolon-after-character-reference 7..7"#,
        ),
        (
            "&Aacute;",
            r#"Character "\u{c1}" 0..8
Eof 8..8"#,
        ),
        (
            "&aacute",
            r#"Character "\u{e1}" 0..7
Eof 7..7
! missing-semicolon-after-character-reference 7..7"#,
        ),
        (
            "&copy=",
            r#"Character "\u{a9}=" 0..6
Eof 6..6
! missing-semicolon-after-character-reference 5..5"#,
        ),
        (
            "&copy;=",
            r#"Character "\u{a9}=" 0..7
Eof 7..7"#,
        ),
        (
            "&#65;",
            r#"Character "A" 0..5
Eof 5..5"#,
        ),
        (
            "&#65",
            r#"Character "A" 0..4
Eof 4..4
! missing-semicolon-after-character-reference 4..4"#,
        ),
        (
            "&#x41;",
            r#"Character "A" 0..6
Eof 6..6"#,
        ),
        (
            "&#X41;",
            r#"Character "A" 0..6
Eof 6..6"#,
        ),
        (
            "&#x41",
            r#"Character "A" 0..5
Eof 5..5
! missing-semicolon-after-character-reference 5..5"#,
        ),
        (
            "&#x41g",
            r#"Character "Ag" 0..6
Eof 6..6
! missing-semicolon-after-character-reference 5..5"#,
        ),
        (
            "&#",
            r#"Character "&#" 0..2
Eof 2..2
! absence-of-digits-in-numeric-character-reference 2..2"#,
        ),
        (
            "&#;",
            r#"Character "&#;" 0..3
Eof 3..3
! absence-of-digits-in-numeric-character-reference 2..2"#,
        ),
        (
            "&#x",
            r#"Character "&#x" 0..3
Eof 3..3
! absence-of-digits-in-numeric-character-reference 3..3"#,
        ),
        (
            "&#x;",
            r#"Character "&#x;" 0..4
Eof 4..4
! absence-of-digits-in-numeric-character-reference 3..3"#,
        ),
        (
            "&#X;",
            r#"Character "&#X;" 0..4
Eof 4..4
! absence-of-digits-in-numeric-character-reference 3..3"#,
        ),
        (
            "&#a",
            r#"Character "&#a" 0..3
Eof 3..3
! absence-of-digits-in-numeric-character-reference 2..2"#,
        ),
        (
            "&#xg",
            r#"Character "&#xg" 0..4
Eof 4..4
! absence-of-digits-in-numeric-character-reference 3..3"#,
        ),
        (
            "&#0;",
            r#"Character "\u{fffd}" 0..4
Eof 4..4
! null-character-reference 4..4"#,
        ),
        (
            "&#00000000065;",
            r#"Character "A" 0..14
Eof 14..14"#,
        ),
        (
            "&#x0;",
            r#"Character "\u{fffd}" 0..5
Eof 5..5
! null-character-reference 5..5"#,
        ),
        (
            "&#128;",
            r#"Character "\u{20ac}" 0..6
Eof 6..6
! control-character-reference 6..6"#,
        ),
        (
            "&#x80;",
            r#"Character "\u{20ac}" 0..6
Eof 6..6
! control-character-reference 6..6"#,
        ),
        (
            "&#x81;",
            r#"Character "\u{81}" 0..6
Eof 6..6
! control-character-reference 6..6"#,
        ),
        (
            "&#x8D;",
            r#"Character "\u{8d}" 0..6
Eof 6..6
! control-character-reference 6..6"#,
        ),
        (
            "&#x9F;",
            r#"Character "\u{178}" 0..6
Eof 6..6
! control-character-reference 6..6"#,
        ),
        (
            "&#13;",
            r#"Character "\u{d}" 0..5
Eof 5..5
! control-character-reference 5..5"#,
        ),
        (
            "&#x0D;",
            r#"Character "\u{d}" 0..6
Eof 6..6
! control-character-reference 6..6"#,
        ),
        (
            "&#1;",
            r#"Character "\u{1}" 0..4
Eof 4..4
! control-character-reference 4..4"#,
        ),
        (
            "&#x7F;",
            r#"Character "\u{7f}" 0..6
Eof 6..6
! control-character-reference 6..6"#,
        ),
        (
            "&#xD800;",
            r#"Character "\u{fffd}" 0..8
Eof 8..8
! surrogate-character-reference 8..8"#,
        ),
        (
            "&#xDFFF;",
            r#"Character "\u{fffd}" 0..8
Eof 8..8
! surrogate-character-reference 8..8"#,
        ),
        (
            "&#x110000;",
            r#"Character "\u{fffd}" 0..10
Eof 10..10
! character-reference-outside-unicode-range 10..10"#,
        ),
        (
            "&#xFFFFFFFFFFFFFFFFFFFF;",
            r#"Character "\u{fffd}" 0..24
Eof 24..24
! character-reference-outside-unicode-range 24..24"#,
        ),
        (
            "&#99999999999999999999;",
            r#"Character "\u{fffd}" 0..23
Eof 23..23
! character-reference-outside-unicode-range 23..23"#,
        ),
        (
            "&#xFDD0;",
            r#"Character "\u{fdd0}" 0..8
Eof 8..8
! noncharacter-character-reference 8..8"#,
        ),
        (
            "&#xFFFE;",
            r#"Character "\u{fffe}" 0..8
Eof 8..8
! noncharacter-character-reference 8..8"#,
        ),
        (
            "&#x1FFFF;",
            r#"Character "\u{1ffff}" 0..9
Eof 9..9
! noncharacter-character-reference 9..9"#,
        ),
        (
            "&#x10FFFF;",
            r#"Character "\u{10ffff}" 0..10
Eof 10..10
! noncharacter-character-reference 10..10"#,
        ),
        (
            "&#9;",
            r#"Whitespace "\u{9}" 0..4
Eof 4..4"#,
        ),
        (
            "&#10;",
            r#"Whitespace "\u{a}" 0..5
Eof 5..5"#,
        ),
        (
            "&#32;",
            r#"Whitespace " " 0..5
Eof 5..5"#,
        ),
        (
            "&#x20;a",
            r#"Whitespace " " 0..6
Character "a" 6..7
Eof 7..7"#,
        ),
        (
            " &#32;",
            r#"Whitespace "  " 0..6
Eof 6..6"#,
        ),
        (
            " &amp; ",
            r#"Whitespace " " 0..5
Character "&" 5..6
Whitespace " " 6..7
Eof 7..7"#,
        ),
        (
            "a&amp;b",
            r#"Character "a&b" 0..7
Eof 7..7"#,
        ),
        (
            "\u{0}&amp;\u{0}",
            r#"Null "\u{0}" 0..5
Character "&" 5..6
Null "\u{0}" 6..7
Eof 7..7
! unexpected-null-character 0..0
! unexpected-null-character 6..6"#,
        ),
        (
            "&#x1F600;",
            r#"Character "\u{1f600}" 0..9
Eof 9..9"#,
        ),
        (
            "&#128512",
            r#"Character "\u{1f600}" 0..8
Eof 8..8
! missing-semicolon-after-character-reference 8..8"#,
        ),
        (
            "&amp\u{d}\u{a}",
            r#"Character "&" 0..4
Whitespace "\u{a}" 4..6
Eof 6..6
! missing-semicolon-after-character-reference 4..4"#,
        ),
        (
            "&\u{d}\u{a}",
            r#"Character "&" 0..1
Whitespace "\u{a}" 1..3
Eof 3..3"#,
        ),
        (
            "&\u{a}",
            r#"Character "&" 0..1
Whitespace "\u{a}" 1..2
Eof 2..2"#,
        ),
        (
            "&#\u{d}",
            r#"Character "&#" 0..2
Whitespace "\u{a}" 2..3
Eof 3..3
! absence-of-digits-in-numeric-character-reference 2..2"#,
        ),
        (
            "&\u{1f600}",
            r#"Character "&\u{1f600}" 0..5
Eof 5..5"#,
        ),
        (
            "&a\u{1f600}",
            r#"Character "&a\u{1f600}" 0..6
Eof 6..6"#,
        ),
        (
            "&#x\u{1f600}",
            r#"Character "&#x\u{1f600}" 0..7
Eof 7..7
! absence-of-digits-in-numeric-character-reference 3..3"#,
        ),
        (
            "a&",
            r#"Character "a&" 0..2
Eof 2..2"#,
        ),
        (
            "&am",
            r#"Character "&am" 0..3
Eof 3..3"#,
        ),
        (
            "&ampamp",
            r#"Character "&amp" 0..7
Eof 7..7
! missing-semicolon-after-character-reference 4..4"#,
        ),
        (
            "&amp;amp;",
            r#"Character "&amp;" 0..9
Eof 9..9"#,
        ),
        (
            "&abcdefghijklmnopqrstuvwxyz",
            r#"Character "&abcdefghijklmnopqrstuvwxyz" 0..27
Eof 27..27"#,
        ),
        (
            "&#xZ",
            r#"Character "&#xZ" 0..4
Eof 4..4
! absence-of-digits-in-numeric-character-reference 3..3"#,
        ),
        (
            "&CounterClockwiseContourIntegral;",
            r#"Character "\u{2233}" 0..33
Eof 33..33"#,
        ),
        (
            "&CounterClockwiseContourIntegra",
            r#"Character "&CounterClockwiseContourIntegra" 0..31
Eof 31..31"#,
        ),
        (
            "&nbsp",
            r#"Character "\u{a0}" 0..5
Eof 5..5
! missing-semicolon-after-character-reference 5..5"#,
        ),
        (
            "&nbspx",
            r#"Character "\u{a0}x" 0..6
Eof 6..6
! missing-semicolon-after-character-reference 5..5"#,
        ),
        (
            "&nbsp;x",
            r#"Character "\u{a0}x" 0..7
Eof 7..7"#,
        ),
        (
            "&lt",
            r#"Character "<" 0..3
Eof 3..3
! missing-semicolon-after-character-reference 3..3"#,
        ),
        (
            "&ltx;",
            r#"Character "<x;" 0..5
Eof 5..5
! missing-semicolon-after-character-reference 3..3"#,
        ),
        (
            "&1",
            r#"Character "&1" 0..2
Eof 2..2"#,
        ),
        (
            "&a1;",
            r#"Character "&a1;" 0..4
Eof 4..4
! unknown-named-character-reference 3..3"#,
        ),
        (
            "&a;b",
            r#"Character "&a;b" 0..4
Eof 4..4
! unknown-named-character-reference 2..2"#,
        ),
        (
            " &a;",
            r#"Whitespace " " 0..1
Character "&a;" 1..4
Eof 4..4
! unknown-named-character-reference 3..3"#,
        ),
        (
            "&abc;x",
            r#"Character "&abc;x" 0..6
Eof 6..6
! unknown-named-character-reference 4..4"#,
        ),
        (
            "&ZZZ;",
            r#"Character "&ZZZ;" 0..5
Eof 5..5
! unknown-named-character-reference 4..4"#,
        ),
        (
            "&#65;&#66;&#67;",
            r#"Character "ABC" 0..15
Eof 15..15"#,
        ),
        (
            "&#9;&#10;&#32;a",
            r#"Whitespace "\u{9}\u{a} " 0..14
Character "a" 14..15
Eof 15..15"#,
        ),
        (
            "&#0",
            r#"Character "\u{fffd}" 0..3
Eof 3..3
! missing-semicolon-after-character-reference 3..3
! null-character-reference 3..3"#,
        ),
        (
            "&#x0",
            r#"Character "\u{fffd}" 0..4
Eof 4..4
! missing-semicolon-after-character-reference 4..4
! null-character-reference 4..4"#,
        ),
        (
            "<title>&amp;&amp&ampx&#65&#x;</title>",
            r#"StartTag "title" 0..7
Character "&&&xA&#x;" 7..29
EndTag "title" 29..37
Eof 37..37
! missing-semicolon-after-character-reference 16..16
! missing-semicolon-after-character-reference 20..20
! missing-semicolon-after-character-reference 25..25
! absence-of-digits-in-numeric-character-reference 28..28"#,
        ),
        (
            "<title>&notit;&xyz;</title>",
            r#"StartTag "title" 0..7
Character "\u{ac}it;&xyz;" 7..19
EndTag "title" 19..27
Eof 27..27
! missing-semicolon-after-character-reference 11..11
! unknown-named-character-reference 18..18"#,
        ),
        (
            "<textarea>&#0;&#128;</textarea>",
            r#"StartTag "textarea" 0..10
Character "\u{fffd}\u{20ac}" 10..20
EndTag "textarea" 20..31
Eof 31..31
! null-character-reference 14..14
! control-character-reference 20..20"#,
        ),
    ]);
}

#[test]
fn tokenizes_references_in_attributes_as_parse5() {
    check(&[
        (
            "<a b=\"&amp;\">",
            r#"StartTag "a" 0..13 "b"="&"@3..12
Eof 13..13"#,
        ),
        (
            "<a b=\"&amp\">",
            r#"StartTag "a" 0..12 "b"="&"@3..11
Eof 12..12
! missing-semicolon-after-character-reference 10..10"#,
        ),
        (
            "<a b=\"&ampx\">",
            r#"StartTag "a" 0..13 "b"="&ampx"@3..12
Eof 13..13"#,
        ),
        (
            "<a b=\"&amp=\">",
            r#"StartTag "a" 0..13 "b"="&amp="@3..12
Eof 13..13"#,
        ),
        (
            "<a b=\"&amp-\">",
            r#"StartTag "a" 0..13 "b"="&-"@3..12
Eof 13..13
! missing-semicolon-after-character-reference 10..10"#,
        ),
        (
            "<a b=\"&amp;x\">",
            r#"StartTag "a" 0..14 "b"="&x"@3..13
Eof 14..14"#,
        ),
        (
            "<a b=&amp>",
            r#"StartTag "a" 0..10 "b"="&"@3..9
Eof 10..10
! missing-semicolon-after-character-reference 9..9"#,
        ),
        (
            "<a b=&ampx>",
            r#"StartTag "a" 0..11 "b"="&ampx"@3..10
Eof 11..11"#,
        ),
        (
            "<a b=&amp=>",
            r#"StartTag "a" 0..11 "b"="&amp="@3..10
Eof 11..11
! unexpected-character-in-unquoted-attribute-value 9..9"#,
        ),
        (
            "<a b=\"&notit;\">",
            r#"StartTag "a" 0..15 "b"="&notit;"@3..14
Eof 15..15"#,
        ),
        (
            "<a b=\"&notin;\">",
            r#"StartTag "a" 0..15 "b"="\u{2209}"@3..14
Eof 15..15"#,
        ),
        (
            "<a b=\"&noti\">",
            r#"StartTag "a" 0..13 "b"="&noti"@3..12
Eof 13..13"#,
        ),
        (
            "<a b=\"&not\">",
            r#"StartTag "a" 0..12 "b"="\u{ac}"@3..11
Eof 12..12
! missing-semicolon-after-character-reference 10..10"#,
        ),
        (
            "<a b='&not'>",
            r#"StartTag "a" 0..12 "b"="\u{ac}"@3..11
Eof 12..12
! missing-semicolon-after-character-reference 10..10"#,
        ),
        (
            "<a b=\"&xyz;\">",
            r#"StartTag "a" 0..13 "b"="&xyz;"@3..12
Eof 13..13"#,
        ),
        (
            "<a b=\"&;\">",
            r#"StartTag "a" 0..10 "b"="&;"@3..9
Eof 10..10"#,
        ),
        (
            "<a b=\"&\">",
            r#"StartTag "a" 0..9 "b"="&"@3..8
Eof 9..9"#,
        ),
        (
            "<a b=&>",
            r#"StartTag "a" 0..7 "b"="&"@3..6
Eof 7..7"#,
        ),
        (
            "<a b=\"&#65\">",
            r#"StartTag "a" 0..12 "b"="A"@3..11
Eof 12..12
! missing-semicolon-after-character-reference 10..10"#,
        ),
        (
            "<a b=\"&#x41;\">",
            r#"StartTag "a" 0..14 "b"="A"@3..13
Eof 14..14"#,
        ),
        (
            "<a b=\"&#;\">",
            r#"StartTag "a" 0..11 "b"="&#;"@3..10
Eof 11..11
! absence-of-digits-in-numeric-character-reference 8..8"#,
        ),
        (
            "<a b=\"&#0;\">",
            r#"StartTag "a" 0..12 "b"="\u{fffd}"@3..11
Eof 12..12
! null-character-reference 10..10"#,
        ),
        (
            "<a b=\"&#x110000;\">",
            r#"StartTag "a" 0..18 "b"="\u{fffd}"@3..17
Eof 18..18
! character-reference-outside-unicode-range 16..16"#,
        ),
        (
            "<a b=\"&#128;\">",
            r#"StartTag "a" 0..14 "b"="\u{20ac}"@3..13
Eof 14..14
! control-character-reference 12..12"#,
        ),
        (
            "<a b=\"&a1;\">",
            r#"StartTag "a" 0..12 "b"="&a1;"@3..11
Eof 12..12"#,
        ),
        (
            "<a b=\"&Aacute\">",
            r#"StartTag "a" 0..15 "b"="\u{c1}"@3..14
Eof 15..15
! missing-semicolon-after-character-reference 13..13"#,
        ),
        (
            "<a b=\"&Aacutex\">",
            r#"StartTag "a" 0..16 "b"="&Aacutex"@3..15
Eof 16..16"#,
        ),
        (
            "<a b=\"&lt",
            r#"Eof 9..9
! missing-semicolon-after-character-reference 9..9
! eof-in-tag 9..9"#,
        ),
        (
            "<a b=\"&notin",
            r#"Eof 12..12
! eof-in-tag 12..12"#,
        ),
        (
            "<a b=\"&noti",
            r#"Eof 11..11
! eof-in-tag 11..11"#,
        ),
        (
            "<a b=&lt",
            r#"Eof 8..8
! missing-semicolon-after-character-reference 8..8
! eof-in-tag 8..8"#,
        ),
        (
            "<a b=&#65",
            r#"Eof 9..9
! missing-semicolon-after-character-reference 9..9
! eof-in-tag 9..9"#,
        ),
        (
            "<a b=\"&#",
            r#"Eof 8..8
! absence-of-digits-in-numeric-character-reference 8..8
! eof-in-tag 8..8"#,
        ),
        (
            "<a b=\"&amp;&amp&ampx&#65&#x;\">",
            r#"StartTag "a" 0..30 "b"="&&&ampxA&#x;"@3..29
Eof 30..30
! missing-semicolon-after-character-reference 15..15
! missing-semicolon-after-character-reference 24..24
! absence-of-digits-in-numeric-character-reference 27..27"#,
        ),
        (
            "<a b=1 b=\"&amp;&#0;\">",
            r#"StartTag "a" 0..21 "b"="1"@3..6
Eof 21..21
! duplicate-attribute 8..8
! null-character-reference 19..19"#,
        ),
        (
            "<a b=\"&NotEqualTilde;\">",
            r#"StartTag "a" 0..23 "b"="\u{2242}\u{338}"@3..22
Eof 23..23"#,
        ),
        (
            "<a b=\"a&amp;b\"c>",
            r#"StartTag "a" 0..16 "b"="a&b"@3..4 "c"=""@14..15
Eof 16..16
! missing-whitespace-between-attributes 14..14"#,
        ),
        (
            "<a b='&#x41&#x42'>",
            r#"StartTag "a" 0..18 "b"="AB"@3..17
Eof 18..18
! missing-semicolon-after-character-reference 11..11
! missing-semicolon-after-character-reference 16..16"#,
        ),
        (
            "<a b=\"&\u{0}\">",
            r#"StartTag "a" 0..10 "b"="&\u{fffd}"@3..9
Eof 10..10
! unexpected-null-character 7..7"#,
        ),
        (
            "<a b=\"&am\">",
            r#"StartTag "a" 0..11 "b"="&am"@3..10
Eof 11..11"#,
        ),
    ]);
}

#[test]
fn tokenizes_eof_in_states_as_parse5() {
    check(&[
        (
            "<",
            r#"Character "<" 0..1
Eof 1..1
! eof-before-tag-name 1..1"#,
        ),
        (
            "</",
            r#"Character "</" 0..2
Eof 2..2
! eof-before-tag-name 2..2"#,
        ),
        (
            "<a",
            r#"Eof 2..2
! eof-in-tag 2..2"#,
        ),
        (
            "<a ",
            r#"Eof 3..3
! eof-in-tag 3..3"#,
        ),
        (
            "<a b",
            r#"Eof 4..4
! eof-in-tag 4..4"#,
        ),
        (
            "<a b ",
            r#"Eof 5..5
! eof-in-tag 5..5"#,
        ),
        (
            "<a b=",
            r#"Eof 5..5
! eof-in-tag 5..5"#,
        ),
        (
            "<a b=\"",
            r#"Eof 6..6
! eof-in-tag 6..6"#,
        ),
        (
            "<a b='",
            r#"Eof 6..6
! eof-in-tag 6..6"#,
        ),
        (
            "<a b=c",
            r#"Eof 6..6
! eof-in-tag 6..6"#,
        ),
        (
            "<a b=\"c\"",
            r#"Eof 8..8
! eof-in-tag 8..8"#,
        ),
        (
            "<a/",
            r#"Eof 3..3
! eof-in-tag 3..3"#,
        ),
        (
            "<a b/",
            r#"Eof 5..5
! eof-in-tag 5..5"#,
        ),
        (
            "<!",
            r#"Comment "" 0..3
Eof 2..2
! incorrectly-opened-comment 2..2"#,
        ),
        (
            "<!-",
            r#"Comment "-" 0..4
Eof 3..3
! incorrectly-opened-comment 2..2"#,
        ),
        (
            "<!--",
            r#"Comment "" 0..5
Eof 4..4
! eof-in-comment 4..4"#,
        ),
        (
            "<!---",
            r#"Comment "" 0..6
Eof 5..5
! eof-in-comment 5..5"#,
        ),
        (
            "<!--a",
            r#"Comment "a" 0..6
Eof 5..5
! eof-in-comment 5..5"#,
        ),
        (
            "<!--a-",
            r#"Comment "a" 0..7
Eof 6..6
! eof-in-comment 6..6"#,
        ),
        (
            "<!--a--",
            r#"Comment "a" 0..8
Eof 7..7
! eof-in-comment 7..7"#,
        ),
        (
            "<!--a--!",
            r#"Comment "a" 0..9
Eof 8..8
! eof-in-comment 8..8"#,
        ),
        (
            "<!--<",
            r#"Comment "<" 0..6
Eof 5..5
! eof-in-comment 5..5"#,
        ),
        (
            "<!--<!",
            r#"Comment "<!" 0..7
Eof 6..6
! eof-in-comment 6..6"#,
        ),
        (
            "<!--<!-",
            r#"Comment "<!" 0..8
Eof 7..7
! eof-in-comment 7..7"#,
        ),
        (
            "<!--<!--",
            r#"Comment "<!" 0..9
Eof 8..8
! eof-in-comment 8..8"#,
        ),
        (
            "<?",
            r#"Comment "?" 0..3
Eof 2..2
! unexpected-question-mark-instead-of-tag-name 1..1"#,
        ),
        (
            "<?a",
            r#"Comment "?a" 0..4
Eof 3..3
! unexpected-question-mark-instead-of-tag-name 1..1"#,
        ),
        (
            "</a",
            r#"Eof 3..3
! eof-in-tag 3..3"#,
        ),
        (
            "</1",
            r#"Comment "1" 0..4
Eof 3..3
! invalid-first-character-of-tag-name 2..2"#,
        ),
        (
            "<!DOCTYPE",
            r#"Doctype null null null 0..10 quirks
Eof 9..9
! eof-in-doctype 9..9"#,
        ),
        (
            "<!DOCTYPE ",
            r#"Doctype null null null 0..11 quirks
Eof 10..10
! eof-in-doctype 10..10"#,
        ),
        (
            "<!DOCTYPE a",
            r#"Doctype "a" null null 0..12 quirks
Eof 11..11
! eof-in-doctype 11..11"#,
        ),
        (
            "<!DOCTYPE a ",
            r#"Doctype "a" null null 0..13 quirks
Eof 12..12
! eof-in-doctype 12..12"#,
        ),
        (
            "<!DOCTYPE a PUBLIC",
            r#"Doctype "a" null null 0..19 quirks
Eof 18..18
! eof-in-doctype 18..18"#,
        ),
        (
            "<!DOCTYPE a PUBLIC ",
            r#"Doctype "a" null null 0..20 quirks
Eof 19..19
! eof-in-doctype 19..19"#,
        ),
        (
            "<!DOCTYPE a PUBLIC \"",
            r#"Doctype "a" "" null 0..21 quirks
Eof 20..20
! eof-in-doctype 20..20"#,
        ),
        (
            "<!DOCTYPE a PUBLIC '",
            r#"Doctype "a" "" null 0..21 quirks
Eof 20..20
! eof-in-doctype 20..20"#,
        ),
        (
            "<!DOCTYPE a PUBLIC \"\"",
            r#"Doctype "a" "" null 0..22 quirks
Eof 21..21
! eof-in-doctype 21..21"#,
        ),
        (
            "<!DOCTYPE a PUBLIC \"\" ",
            r#"Doctype "a" "" null 0..23 quirks
Eof 22..22
! eof-in-doctype 22..22"#,
        ),
        (
            "<!DOCTYPE a PUBLIC \"\" \"",
            r#"Doctype "a" "" "" 0..24 quirks
Eof 23..23
! eof-in-doctype 23..23"#,
        ),
        (
            "<!DOCTYPE a PUBLIC \"\" '",
            r#"Doctype "a" "" "" 0..24 quirks
Eof 23..23
! eof-in-doctype 23..23"#,
        ),
        (
            "<!DOCTYPE a PUBLIC \"\" \"\"",
            r#"Doctype "a" "" "" 0..25 quirks
Eof 24..24
! eof-in-doctype 24..24"#,
        ),
        (
            "<!DOCTYPE a PUBLIC \"\" \"\" ",
            r#"Doctype "a" "" "" 0..26 quirks
Eof 25..25
! eof-in-doctype 25..25"#,
        ),
        (
            "<!DOCTYPE a SYSTEM",
            r#"Doctype "a" null null 0..19 quirks
Eof 18..18
! eof-in-doctype 18..18"#,
        ),
        (
            "<!DOCTYPE a SYSTEM ",
            r#"Doctype "a" null null 0..20 quirks
Eof 19..19
! eof-in-doctype 19..19"#,
        ),
        (
            "<!DOCTYPE a SYSTEM \"",
            r#"Doctype "a" null "" 0..21 quirks
Eof 20..20
! eof-in-doctype 20..20"#,
        ),
        (
            "<!DOCTYPE a x",
            r#"Doctype "a" null null 0..14 quirks
Eof 13..13
! invalid-character-sequence-after-doctype-name 12..12"#,
        ),
        (
            "&",
            r#"Character "&" 0..1
Eof 1..1"#,
        ),
        (
            "&#",
            r#"Character "&#" 0..2
Eof 2..2
! absence-of-digits-in-numeric-character-reference 2..2"#,
        ),
        (
            "&#x",
            r#"Character "&#x" 0..3
Eof 3..3
! absence-of-digits-in-numeric-character-reference 3..3"#,
        ),
        (
            "&#1",
            r#"Character "\u{1}" 0..3
Eof 3..3
! missing-semicolon-after-character-reference 3..3
! control-character-reference 3..3"#,
        ),
        (
            "&#x1",
            r#"Character "\u{1}" 0..4
Eof 4..4
! missing-semicolon-after-character-reference 4..4
! control-character-reference 4..4"#,
        ),
        (
            "&a",
            r#"Character "&a" 0..2
Eof 2..2"#,
        ),
        (
            "&amp",
            r#"Character "&" 0..4
Eof 4..4
! missing-semicolon-after-character-reference 4..4"#,
        ),
        (
            "&notin",
            r#"Character "\u{ac}in" 0..6
Eof 6..6
! missing-semicolon-after-character-reference 4..4"#,
        ),
        (
            "a\u{d}",
            r#"Character "a" 0..1
Whitespace "\u{a}" 1..2
Eof 2..2"#,
        ),
        (
            "<a\u{d}",
            r#"Eof 3..3
! eof-in-tag 3..3"#,
        ),
        (
            "<a b=\"\u{d}",
            r#"Eof 7..7
! eof-in-tag 7..7"#,
        ),
    ]);
}

#[test]
fn tokenizes_hostile_as_parse5() {
    check(&[
        (
            "<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<",
            r#"Character "<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<" 0..50
Eof 50..50
! invalid-first-character-of-tag-name 1..1
! invalid-first-character-of-tag-name 2..2
! invalid-first-character-of-tag-name 3..3
! invalid-first-character-of-tag-name 4..4
! invalid-first-character-of-tag-name 5..5
! invalid-first-character-of-tag-name 6..6
! invalid-first-character-of-tag-name 7..7
! invalid-first-character-of-tag-name 8..8
! invalid-first-character-of-tag-name 9..9
! invalid-first-character-of-tag-name 10..10
! invalid-first-character-of-tag-name 11..11
! invalid-first-character-of-tag-name 12..12
! invalid-first-character-of-tag-name 13..13
! invalid-first-character-of-tag-name 14..14
! invalid-first-character-of-tag-name 15..15
! invalid-first-character-of-tag-name 16..16
! invalid-first-character-of-tag-name 17..17
! invalid-first-character-of-tag-name 18..18
! invalid-first-character-of-tag-name 19..19
! invalid-first-character-of-tag-name 20..20
! invalid-first-character-of-tag-name 21..21
! invalid-first-character-of-tag-name 22..22
! invalid-first-character-of-tag-name 23..23
! invalid-first-character-of-tag-name 24..24
! invalid-first-character-of-tag-name 25..25
! invalid-first-character-of-tag-name 26..26
! invalid-first-character-of-tag-name 27..27
! invalid-first-character-of-tag-name 28..28
! invalid-first-character-of-tag-name 29..29
! invalid-first-character-of-tag-name 30..30
! invalid-first-character-of-tag-name 31..31
! invalid-first-character-of-tag-name 32..32
! invalid-first-character-of-tag-name 33..33
! invalid-first-character-of-tag-name 34..34
! invalid-first-character-of-tag-name 35..35
! invalid-first-character-of-tag-name 36..36
! invalid-first-character-of-tag-name 37..37
! invalid-first-character-of-tag-name 38..38
! invalid-first-character-of-tag-name 39..39
! invalid-first-character-of-tag-name 40..40
! invalid-first-character-of-tag-name 41..41
! invalid-first-character-of-tag-name 42..42
! invalid-first-character-of-tag-name 43..43
! invalid-first-character-of-tag-name 44..44
! invalid-first-character-of-tag-name 45..45
! invalid-first-character-of-tag-name 46..46
! invalid-first-character-of-tag-name 47..47
! invalid-first-character-of-tag-name 48..48
! invalid-first-character-of-tag-name 49..49
! eof-before-tag-name 50..50"#,
        ),
        (
            "&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&",
            r#"Character "&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&" 0..30
Eof 30..30"#,
        ),
        (
            "&amp&amp&amp&amp&amp&amp&amp&amp&amp&amp&amp&amp&amp&amp&amp&amp&amp&amp&amp&amp",
            r#"Character "&&&&&&&&&&&&&&&&&&&&" 0..80
Eof 80..80
! missing-semicolon-after-character-reference 4..4
! missing-semicolon-after-character-reference 8..8
! missing-semicolon-after-character-reference 12..12
! missing-semicolon-after-character-reference 16..16
! missing-semicolon-after-character-reference 20..20
! missing-semicolon-after-character-reference 24..24
! missing-semicolon-after-character-reference 28..28
! missing-semicolon-after-character-reference 32..32
! missing-semicolon-after-character-reference 36..36
! missing-semicolon-after-character-reference 40..40
! missing-semicolon-after-character-reference 44..44
! missing-semicolon-after-character-reference 48..48
! missing-semicolon-after-character-reference 52..52
! missing-semicolon-after-character-reference 56..56
! missing-semicolon-after-character-reference 60..60
! missing-semicolon-after-character-reference 64..64
! missing-semicolon-after-character-reference 68..68
! missing-semicolon-after-character-reference 72..72
! missing-semicolon-after-character-reference 76..76
! missing-semicolon-after-character-reference 80..80"#,
        ),
        (
            "<a <a <a <a <a <a <a <a <a <a ",
            r#"Eof 30..30
! unexpected-character-in-attribute-name 3..3
! unexpected-character-in-attribute-name 6..6
! duplicate-attribute 8..8
! unexpected-character-in-attribute-name 9..9
! duplicate-attribute 11..11
! unexpected-character-in-attribute-name 12..12
! duplicate-attribute 14..14
! unexpected-character-in-attribute-name 15..15
! duplicate-attribute 17..17
! unexpected-character-in-attribute-name 18..18
! duplicate-attribute 20..20
! unexpected-character-in-attribute-name 21..21
! duplicate-attribute 23..23
! unexpected-character-in-attribute-name 24..24
! duplicate-attribute 26..26
! unexpected-character-in-attribute-name 27..27
! duplicate-attribute 29..29
! eof-in-tag 30..30"#,
        ),
        (
            "<a b b b b b b b b b b b b b b b b b b b b >",
            r#"StartTag "a" 0..44 "b"=""@3..4
Eof 44..44
! duplicate-attribute 6..6
! duplicate-attribute 8..8
! duplicate-attribute 10..10
! duplicate-attribute 12..12
! duplicate-attribute 14..14
! duplicate-attribute 16..16
! duplicate-attribute 18..18
! duplicate-attribute 20..20
! duplicate-attribute 22..22
! duplicate-attribute 24..24
! duplicate-attribute 26..26
! duplicate-attribute 28..28
! duplicate-attribute 30..30
! duplicate-attribute 32..32
! duplicate-attribute 34..34
! duplicate-attribute 36..36
! duplicate-attribute 38..38
! duplicate-attribute 40..40
! duplicate-attribute 42..42"#,
        ),
        (
            "</</</</</</</</</</</</</</</</</</</</",
            r#"Comment "</</</</</</</</</</</</</</</</</</</" 0..41
Eof 40..40
! invalid-first-character-of-tag-name 2..2"#,
        ),
        (
            "<!<!<!<!<!<!<!<!<!<!",
            r#"Comment "<!<!<!<!<!<!<!<!<!" 0..21
Eof 20..20
! incorrectly-opened-comment 2..2"#,
        ),
        (
            "<!--<!--<!--<!--<!--<!--<!--<!--<!--<!--",
            r#"Comment "<!--<!--<!--<!--<!--<!--<!--<!--<!" 0..41
Eof 40..40
! nested-comment 8..8
! nested-comment 12..12
! nested-comment 16..16
! nested-comment 20..20
! nested-comment 24..24
! nested-comment 28..28
! nested-comment 32..32
! nested-comment 36..36
! eof-in-comment 40..40"#,
        ),
        (
            "-->-->-->-->-->-->-->-->-->-->",
            r#"Character "-->-->-->-->-->-->-->-->-->-->" 0..30
Eof 30..30"#,
        ),
        (
            "&#&#&#&#&#&#&#&#&#&#",
            r#"Character "&#&#&#&#&#&#&#&#&#&#" 0..20
Eof 20..20
! absence-of-digits-in-numeric-character-reference 2..2
! absence-of-digits-in-numeric-character-reference 4..4
! absence-of-digits-in-numeric-character-reference 6..6
! absence-of-digits-in-numeric-character-reference 8..8
! absence-of-digits-in-numeric-character-reference 10..10
! absence-of-digits-in-numeric-character-reference 12..12
! absence-of-digits-in-numeric-character-reference 14..14
! absence-of-digits-in-numeric-character-reference 16..16
! absence-of-digits-in-numeric-character-reference 18..18
! absence-of-digits-in-numeric-character-reference 20..20"#,
        ),
        (
            "<!--------------------------------",
            r#"Comment "----------------------------" 0..35
Eof 34..34
! eof-in-comment 34..34"#,
        ),
        (
            "<!----!--!--!--!--!--!--!--!--!--!",
            r#"Comment "--!--!--!--!--!--!--!--!--!" 0..35
Eof 34..34
! eof-in-comment 34..34"#,
        ),
        (
            "<!DOCTYPE<!DOCTYPE<!DOCTYPE<!DOCTYPE<!DOCTYPE",
            r#"Doctype "<!doctype<!doctype<!doctype<!doctype" null null 0..46 quirks
Eof 45..45
! missing-whitespace-before-doctype-name 9..9
! eof-in-doctype 45..45"#,
        ),
        (
            "]]>]]>]]>]]>]]>",
            r#"Character "]]>]]>]]>]]>]]>" 0..15
Eof 15..15"#,
        ),
        (
            "<svg><![CDATA[]]]]]]]]]]]]]]]]]]]]",
            r#"StartTag "svg" 0..5
Character "]]]]]]]]]]]]]]]]]]]]" 5..34
Eof 34..34
! eof-in-cdata 34..34"#,
        ),
        (
            "&#x&#x&#x&#x&#x111111111111111111111111111111;",
            r#"Character "&#x&#x&#x&#x\u{fffd}" 0..46
Eof 46..46
! absence-of-digits-in-numeric-character-reference 3..3
! absence-of-digits-in-numeric-character-reference 6..6
! absence-of-digits-in-numeric-character-reference 9..9
! absence-of-digits-in-numeric-character-reference 12..12
! character-reference-outside-unicode-range 46..46"#,
        ),
    ]);
}

#[test]
fn tokenizes_from_a_state_set_before_the_first_token_as_parse5() {
    check_initial(&[
        (
            "a</title>b",
            State::Rcdata,
            "title",
            r#"Character "a" 0..1
EndTag "title" 1..9
Character "b" 9..10
Eof 10..10"#,
        ),
        (
            "a<b>&amp;</textarea>",
            State::Rcdata,
            "textarea",
            r#"Character "a<b>&" 0..9
EndTag "textarea" 9..20
Eof 20..20"#,
        ),
        (
            "a</style>",
            State::Rawtext,
            "style",
            r#"Character "a" 0..1
EndTag "style" 1..9
Eof 9..9"#,
        ),
        (
            "a<!--</script>",
            State::ScriptData,
            "script",
            r#"Character "a<!--" 0..5
EndTag "script" 5..14
Eof 14..14"#,
        ),
        (
            "a</plaintext>",
            State::Plaintext,
            "",
            r#"Character "a</plaintext>" 0..13
Eof 13..13"#,
        ),
        (
            "a]]>b",
            State::CdataSection,
            "",
            r#"Character "ab" 0..5
Eof 5..5"#,
        ),
        (
            "a</x>",
            State::Rcdata,
            "",
            r#"Character "a</x>" 0..5
Eof 5..5"#,
        ),
        (
            "a</title>",
            State::Rcdata,
            "TITLE",
            r#"Character "a</title>" 0..9
Eof 9..9"#,
        ),
        (
            "a</title_>",
            State::Rcdata,
            "title_",
            r#"Character "a</title_>" 0..10
Eof 10..10"#,
        ),
        (
            "a</\u{e4}>",
            State::Rcdata,
            "\u{e4}",
            r#"Character "a</\u{e4}>" 0..6
Eof 6..6"#,
        ),
        (
            "a</\u{c4}>",
            State::Rcdata,
            "\u{e4}",
            r#"Character "a</\u{c4}>" 0..6
Eof 6..6"#,
        ),
        (
            "</\u{1f600}>",
            State::Rcdata,
            "\u{1f600}",
            r#"Character "</\u{1f600}>" 0..7
Eof 7..7"#,
        ),
        (
            "a</@>",
            State::Rawtext,
            "@",
            r#"Character "a</@>" 0..5
Eof 5..5"#,
        ),
    ]);
}
