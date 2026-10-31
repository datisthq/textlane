use textlane_core::{Allocator, Error, File};

use super::mark_math;
use crate::actions::hast::stringify::stringify_hast;
use crate::actions::html::parse::parse_html;
use crate::models::options::{Math, Parse, Stringify};

fn mark(input: &str, options: &Math) -> Result<String, Error> {
    let alloc = Allocator::new();
    let parse = Parse {
        fragment: true,
        ..Default::default()
    };
    let mut node = parse_html(input, &parse, &mut File::new(), &alloc);
    mark_math(&mut node, options, &alloc);
    stringify_hast(&node, &Stringify::default())
}

#[test]
fn marks_inline_and_display_math() -> Result<(), Error> {
    assert_eq!(
        mark("<p>a $x_1$ b $$\\frac{a}{b}$$ c</p>", &Math::default())?,
        "<p>a <code class=\"language-math math-inline\">x_1</code> b \
         <code class=\"language-math math-display\">\\frac{a}{b}</code> c</p>"
    );
    Ok(())
}

#[test]
fn keeps_prices_and_escaped_dollars_as_text() -> Result<(), Error> {
    let input = "<p>$20,000 and $30,000</p><p>costs $ 5$</p><p>\\$x$</p>";
    assert_eq!(
        mark(input, &Math::default())?,
        "<p>$20,000 and $30,000</p><p>costs $ 5$</p><p>$x$</p>"
    );
    Ok(())
}

#[test]
fn leaves_ignored_elements_alone() -> Result<(), Error> {
    let options = Math {
        ignore_attributes: vec!["data-literal".into()],
        ..Math::default()
    };
    let input = "<code>$a$</code><a>$b$</a><p data-literal>$c$</p><p>$d$</p>";
    assert_eq!(
        mark(input, &options)?,
        "<code>$a$</code><a>$b$</a><p data-literal=\"\">$c$</p>\
         <p><code class=\"language-math math-inline\">d</code></p>"
    );
    Ok(())
}

#[test]
fn takes_only_double_dollars_without_single_dollar_math() -> Result<(), Error> {
    let options = Math {
        single_dollar_text_math: false,
        ..Math::default()
    };
    assert_eq!(
        mark("<p>$a$ $$b$$</p>", &options)?,
        "<p>$a$ <code class=\"language-math math-display\">b</code></p>"
    );
    Ok(())
}
