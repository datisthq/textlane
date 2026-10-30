use serde_json::json;
use textlane_core::{Allocator, Error, File};

use super::typeset_hast;
use crate::actions::hast::stringify::stringify_hast;
use crate::actions::html::parse::parse_html;
use crate::models::options::{Katex, Parse, Stringify};

fn typeset(input: &str, options: serde_json::Value) -> Result<(String, File), Error> {
    let alloc = Allocator::new();
    let options: Katex = serde_json::from_value(options)
        .map_err(|error| Error::new("invalid-options", error.to_string()))?;
    let parse = Parse {
        fragment: true,
        ..Default::default()
    };
    let mut file = File::new();
    file.value = Some(input.to_string());
    let mut node = parse_html(input, &parse, &mut file, &alloc);
    typeset_hast(&mut node, &options, &mut file, &alloc);
    Ok((stringify_hast(&node, &Stringify::default())?, file))
}

#[test]
fn renders_inline_math_in_place() -> Result<(), Error> {
    let (html, file) = typeset(
        "<p>a <code class=\"language-math math-inline\">x</code> b</p>",
        json!({}),
    )?;
    assert!(html.starts_with("<p>a <span class=\"katex\"><span class=\"katex-mathml\">"));
    assert!(html.ends_with("</span></span></span> b</p>"));
    assert!(!html.contains("katex-display"));
    assert!(file.messages.is_empty());
    Ok(())
}

#[test]
fn replaces_the_pre_of_a_math_fence_in_display_mode() -> Result<(), Error> {
    let (html, _) = typeset(
        "<pre><code class=\"language-math math-display\">x</code></pre>",
        json!({}),
    )?;
    assert!(html.starts_with("<span class=\"katex-display\"><span class=\"katex\">"));
    assert!(!html.contains("<pre>"));
    Ok(())
}

#[test]
fn renders_the_same_markup_on_every_run() -> Result<(), Error> {
    let input =
        "<code class=\"math-inline\">\\sqrt{x} \\xrightarrow{a} \\rule{1em}{2em}</code>";
    let (first, _) = typeset(input, json!({}))?;
    for _ in 0..8 {
        assert_eq!(typeset(input, json!({}))?.0, first);
    }
    Ok(())
}

#[test]
fn takes_html_only_output_and_macros() -> Result<(), Error> {
    let options = json!({ "output": "html", "macros": { "\\RR": "\\mathbb{R}" } });
    let (html, _) = typeset("<code class=\"math-inline\">\\RR</code>", options)?;
    assert!(!html.contains("katex-mathml"));
    assert!(html.contains("mathbb"));
    Ok(())
}

#[test]
fn reports_a_parse_error_and_renders_it() -> Result<(), Error> {
    let (html, file) =
        typeset("<code class=\"math-inline\">\\frac{1}{</code>", json!({}))?;
    assert!(html.contains("katex-error"));
    assert_eq!(file.messages.len(), 1);
    let message = &file.messages[0];
    assert_eq!(message.reason, "Could not render math with KaTeX");
    assert_eq!(message.source.as_deref(), Some("rehype-katex"));
    assert_eq!(message.rule.as_deref(), Some("parseerror"));
    assert_eq!(
        message.place.as_ref().map(|place| place.start.column),
        Some(1)
    );
    Ok(())
}

#[test]
fn leaves_other_elements_alone() -> Result<(), Error> {
    let input = "<pre><code class=\"language-js\">$x$</code></pre>";
    assert_eq!(typeset(input, json!({}))?.0, input);
    Ok(())
}

#[test]
fn survives_hostile_input() -> Result<(), Error> {
    let nest = |open: &str, close: &str, depth| {
        format!("{}x{}", open.repeat(depth), close.repeat(depth))
    };
    let values = [
        nest("\\frac{1}{", "}", 950),
        nest("\\left(", "\\right)", 950),
        nest("x^{", "}", 950),
        format!("\\def\\a{{{}}}\\a", nest("{", "}", 8).replace('x', "\\a")),
        "\\def\\a{\\a}\\a".to_string(),
        "x^^^^".to_string(),
        "\\begin{array}".to_string(),
        "\u{0}".to_string(),
    ];
    for value in values {
        let input = format!("<code class=\"math-inline\">{value}</code>");
        typeset(&input, json!({ "macros": { "\\b": "{{{{\\b}}}}" } }))?;
    }
    Ok(())
}

#[test]
fn survives_style_switches_nesting_without_braces() -> Result<(), Error> {
    for switch in ["\\displaystyle ", "\\color{red}", "\\tiny ", "\\bf "] {
        for count in [900, 5000] {
            let value = format!("{}x", switch.repeat(count));
            typeset(
                &format!("<code class=\"math-inline\">{value}</code>"),
                json!({}),
            )?;
        }
    }
    Ok(())
}

#[test]
fn survives_closers_the_parser_never_sees() -> Result<(), Error> {
    let tails = [
        "%}\n",
        "%\r}\n",
        "\\verb|}|",
        "\\verb*!}!",
        "\\verb}a}",
        "\\verb*}a}",
        "]",
    ];
    for tail in tails {
        let value = format!("{}x", format!("{{\\displaystyle {tail}").repeat(60_000));
        typeset(
            &format!("<code class=\"math-inline\">{value}</code>"),
            json!({}),
        )?;
    }
    Ok(())
}

#[test]
fn survives_openers_after_a_literal_percent_or_an_open_verb() -> Result<(), Error> {
    for head in ["\\url{%}", "\\verb|", "\\href{%}{a}", "\\sqrt["] {
        let value = format!("{head}{}x", "{\\displaystyle ".repeat(60_000));
        typeset(
            &format!("<code class=\"math-inline\">{value}</code>"),
            json!({ "trust": true }),
        )?;
    }
    Ok(())
}

#[test]
fn survives_macros_an_earlier_formula_defined() -> Result<(), Error> {
    for body in ["{{{{{{{{\\a}}}}}}}}", "\\displaystyle\\displaystyle\\a"] {
        let input = format!(
            "<code class=\"math-inline\">\\gdef\\a{{{body}}}</code>\
             <code class=\"math-inline\">\\a</code>"
        );
        typeset(&input, json!({}))?;
    }
    Ok(())
}

#[test]
fn reports_math_nested_too_deep_as_katex_js_overflows() -> Result<(), Error> {
    let value = format!("{}x{}", "{".repeat(5000), "}".repeat(5000));
    let (html, file) = typeset(
        &format!("<code class=\"math-inline\">{value}</code>"),
        json!({}),
    )?;
    assert!(html.contains("title=\"RangeError: Maximum call stack size exceeded\""));
    assert_eq!(file.messages[0].rule.as_deref(), Some("rangeerror"));
    Ok(())
}
