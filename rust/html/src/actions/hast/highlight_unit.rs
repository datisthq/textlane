use serde_json::json;
use textlane_core::{Allocator, Error, File};

use super::highlight_hast;
use crate::actions::hast::stringify::stringify_hast;
use crate::actions::html::parse::parse_html;
use crate::models::options::{Highlight, Parse, Stringify};

fn highlight(input: &str, options: serde_json::Value) -> Result<String, Error> {
    let alloc = Allocator::new();
    let options: Highlight = serde_json::from_value(options)
        .map_err(|error| Error::new("invalid-options", error.to_string()))?;
    let parse = Parse {
        fragment: true,
        ..Default::default()
    };
    let mut node = parse_html(input, &parse, &mut File::new(), &alloc);
    highlight_hast(&mut node, &options, &alloc)?;
    stringify_hast(&node, &Stringify::default())
}

#[test]
fn colors_with_one_theme_without_css_variables() -> Result<(), Error> {
    let input = "<pre><code class=\"language-js\">// a\nlet a = \"b\"\n</code></pre>";
    assert_eq!(
        highlight(input, json!({ "theme": "github-light" }))?,
        "<pre class=\"shiki github-light\" style=\"background-color:#fff;color:#24292e\" \
         tabindex=\"0\"><code><span class=\"line\"><span style=\"color:#6A737D\">// a</span>\
         </span>\n<span class=\"line\"><span style=\"color:#D73A49\">let</span><span \
         style=\"color:#24292E\"> a </span><span style=\"color:#D73A49\">=</span><span \
         style=\"color:#032F62\"> \"b\"</span></span></code></pre>"
    );
    Ok(())
}

#[test]
fn puts_the_light_theme_first_whatever_the_order() -> Result<(), Error> {
    let input = "<pre><code class=\"language-js\">let a\n</code></pre>";
    let themes = json!({ "themes": { "dark": "github-dark", "light": "github-light" } });
    assert_eq!(
        highlight(input, themes)?,
        "<pre class=\"shiki shiki-themes github-light github-dark\" \
         style=\"background-color:#fff;--shiki-dark-bg:#24292e;color:#24292e;\
         --shiki-dark:#e1e4e8\" tabindex=\"0\"><code><span class=\"line\"><span \
         style=\"color:#D73A49;--shiki-dark:#F97583\">let</span><span \
         style=\"color:#24292E;--shiki-dark:#E1E4E8\"> a</span></span></code></pre>"
    );
    Ok(())
}

#[test]
fn leaves_plain_text_uncolored_with_a_span_per_line() -> Result<(), Error> {
    let options = json!({ "theme": "github-light", "defaultLanguage": "txt" });
    assert_eq!(
        highlight("<pre><code>a\n\nb</code></pre>", options)?,
        "<pre class=\"shiki github-light\" style=\"background-color:#fff;color:#24292e\" \
         tabindex=\"0\"><code><span class=\"line\"><span>a</span></span>\n<span \
         class=\"line\"><span></span></span>\n<span class=\"line\"><span>b</span></span>\
         </code></pre>"
    );
    Ok(())
}

#[test]
fn fails_on_themes_shiki_would_not_load() {
    let input = "<pre><code class=\"language-js\">a</code></pre>";
    for options in [
        json!({ "themes": { "light": "nope", "dark": "github-dark" } }),
        json!({ "themes": { "dark": "github-dark" } }),
        json!({}),
    ] {
        let error = highlight(input, options).err().map(|error| error.code);
        assert_eq!(error.as_deref(), Some("invalid-options"));
    }
}

#[test]
fn checks_themes_only_once_a_block_is_highlighted() -> Result<(), Error> {
    assert_eq!(highlight("<p>a</p>", json!({}))?, "<p>a</p>");
    Ok(())
}

#[test]
fn runs_transformers_named_through_the_rust_api() -> Result<(), Error> {
    use crate::models::options::{HighlightTransformer, TransformerName};
    let options = Highlight {
        theme: Some("github-light".into()),
        transformers: vec![HighlightTransformer::Name(TransformerName::NotationDiff)],
        ..Default::default()
    };
    let alloc = Allocator::new();
    let parse = Parse {
        fragment: true,
        ..Default::default()
    };
    let input = "<pre><code class=\"language-js\">a // [!code ++]\n</code></pre>";
    let mut node = parse_html(input, &parse, &mut File::new(), &alloc);
    highlight_hast(&mut node, &options, &alloc)?;
    let html = stringify_hast(&node, &Stringify::default())?;
    assert!(html.starts_with("<pre class=\"shiki github-light has-diff\""));
    assert!(html.contains("<span class=\"line diff add\">"));
    assert!(!html.contains("[!code"));
    Ok(())
}

#[test]
fn notations_stay_linear_over_many_lines() -> Result<(), Error> {
    let line = "let a = 1 // [!code ++]\n";
    let input = format!(
        "<pre><code class=\"language-js\">{}</code></pre>",
        line.repeat(20_000)
    );
    let started = std::time::Instant::now();
    let html = highlight(
        &input,
        json!({ "theme": "github-light", "transformers": ["notationDiff"] }),
    )?;
    assert_eq!(html.matches("diff add").count(), 20_000);
    assert!(started.elapsed().as_secs() < 60);
    Ok(())
}

#[test]
fn reads_a_huge_meta_range_without_expanding_it() -> Result<(), Error> {
    let options = json!({ "theme": "github-light", "transformers": ["metaHighlight"] });
    let html = highlight_meta("a\nb", "{2-99999999999999}", options)?;
    assert_eq!(html.matches("highlighted").count(), 1);
    Ok(())
}

fn highlight_meta(
    code: &str,
    meta: &str,
    options: serde_json::Value,
) -> Result<String, Error> {
    let options: Highlight = serde_json::from_value(options)
        .map_err(|error| Error::new("invalid-options", error.to_string()))?;
    let alloc = Allocator::new();
    let input = format!(
        "<pre><code class=\"language-js\" metastring=\"{meta}\">{code}</code></pre>"
    );
    let parse = Parse {
        fragment: true,
        ..Default::default()
    };
    let mut node = parse_html(&input, &parse, &mut File::new(), &alloc);
    highlight_hast(&mut node, &options, &alloc)?;
    stringify_hast(&node, &Stringify::default())
}

#[test]
fn colors_ansi_code_from_its_escapes() -> Result<(), Error> {
    let input = "<pre><code class=\"language-ansi\">\x1b[1;31ma\x1b[0m b</code></pre>";
    assert_eq!(
        highlight(input, json!({ "theme": "github-light" }))?,
        "<pre class=\"shiki github-light\" style=\"background-color:#fff;color:#24292e\" \
         tabindex=\"0\"><code><span class=\"line\"><span \
         style=\"color:#d73a49;font-weight:bold\">a</span><span \
         style=\"color:#24292e\"> b</span></span></code></pre>"
    );
    Ok(())
}

#[test]
fn fails_as_shiki_does_dimming_a_color_past_the_table() {
    let input = "<pre><code class=\"language-ansi\">\x1b[2;38;5;300ma</code></pre>";
    let error = highlight(input, json!({ "theme": "github-light" })).err();
    assert_eq!(
        error.map(|error| error.code).as_deref(),
        Some("highlight-failed")
    );
}

#[test]
fn colors_long_ansi_code_in_linear_time() -> Result<(), Error> {
    let line = "\x1b[32m✓\x1b[39m \x1b[2mok\x1b[22m\n";
    let input = format!(
        "<pre><code class=\"language-ansi\">{}</code></pre>",
        line.repeat(20_000)
    );
    let started = std::time::Instant::now();
    let html = highlight(
        &input,
        json!({ "themes": { "light": "github-light", "dark": "github-dark" } }),
    )?;
    assert_eq!(html.matches("class=\"line\"").count(), 20_000);
    assert!(
        started.elapsed().as_secs() < 20,
        "took {:?}",
        started.elapsed()
    );
    Ok(())
}

#[test]
fn fails_on_meta_words_that_cross() {
    let options =
        json!({ "theme": "github-light", "transformers": ["metaWordHighlight"] });
    let error = highlight_meta("abc", "/ab/ /bc/", options).err();
    assert_eq!(
        error.map(|error| error.code).as_deref(),
        Some("highlight-failed")
    );
}

#[test]
fn highlights_meta_words_on_many_lines_in_linear_time() -> Result<(), Error> {
    let options =
        json!({ "theme": "github-light", "transformers": ["metaWordHighlight"] });
    let code = "let config = config\n".repeat(20_000);
    let started = std::time::Instant::now();
    let html = highlight_meta(code.trim_end(), "/config/", options)?;
    assert_eq!(html.matches("highlighted-word").count(), 40_000);
    assert!(started.elapsed().as_secs() < 60);
    Ok(())
}

#[test]
fn caps_meta_words_on_one_long_line() -> Result<(), Error> {
    let options =
        json!({ "theme": "github-light", "transformers": ["metaWordHighlight"] });
    // plain text skips the grammar, so this times the decorations alone
    let code = "a ".repeat(100_000);
    let input = format!(
        "<pre><code class=\"language-text\" metastring=\"/a/\">{}</code></pre>",
        code.trim_end()
    );
    let started = std::time::Instant::now();
    let html = highlight(&input, options)?;
    assert_eq!(html.matches("highlighted-word").count(), 1_000);
    assert!(started.elapsed().as_secs() < 60);
    Ok(())
}

#[test]
fn leaves_a_long_line_plain_and_keeps_the_rest_colored() -> Result<(), Error> {
    let long = "a ".repeat(50_000);
    let input = format!(
        "<pre><code class=\"language-js\">let a = 1\n{}\nlet b = 2</code></pre>",
        long.trim_end()
    );
    let started = std::time::Instant::now();
    let html = highlight(&input, json!({ "theme": "github-light" }))?;
    assert!(started.elapsed().as_secs() < 5);
    assert!(html.contains("<span class=\"line\"><span>a a a"));
    assert_eq!(html.matches("color:#D73A49\">let</span>").count(), 2);
    Ok(())
}

#[test]
fn marks_lines_words_and_numbers_from_attributes() -> Result<(), Error> {
    let input = "<pre data-title=\"a.js\"><code class=\"language-js\" data-lines=\"2\" \
                 data-focus=\"1,3-3\" data-words=\"b\" data-line-numbers=\"7\">let a\n\
                 let b\nlet c\n</code></pre>";
    let options = json!({
        "theme": "github-light",
        "lineAttributes": {
            "data-lines": "highlighted",
            "data-focus": { "line": "focused", "pre": "has-focused" }
        },
        "wordAttribute": "data-words",
        "lineNumberAttribute": "data-line-numbers",
        "keepProperties": true
    });
    let html = highlight(input, options)?;
    assert!(html.starts_with("<pre class=\"shiki github-light has-focused\""));
    assert!(html.contains("data-title=\"a.js\""));
    assert!(html.contains("<code class=\"language-js\" data-lines=\"2\""));
    assert_eq!(html.matches("class=\"line focused\"").count(), 2);
    assert_eq!(html.matches("class=\"line highlighted\"").count(), 1);
    assert!(html.contains("<span class=\"line-number\">7</span>"));
    assert!(html.contains("<span class=\"line-number\">9</span>"));
    assert_eq!(html.matches("highlighted-word").count(), 1);
    Ok(())
}

#[test]
fn fails_on_lines_it_cannot_read() {
    let input = "<pre><code class=\"language-js\" data-lines=\"2-1\">a\n</code></pre>";
    let options =
        json!({ "theme": "github-light", "lineAttributes": { "data-lines": "x" } });
    let error = highlight(input, options).err().map(|error| error.code);
    assert_eq!(error.as_deref(), Some("invalid-lines"));
}

#[test]
fn colors_inline_code_named_by_class_only_outside_blocks() -> Result<(), Error> {
    let input = "<p><code class=\"language-js\">let a</code></p>\
                 <pre><code class=\"language-nope\">x</code></pre>";
    let html = highlight(input, json!({ "theme": "github-light", "inline": "class" }))?;
    assert!(html.starts_with("<p><span class=\"shiki github-light\""));
    assert!(html.ends_with("<pre><code class=\"language-nope\">x</code></pre>"));
    Ok(())
}

#[test]
fn leaves_blocks_with_elements_alone_when_asked() -> Result<(), Error> {
    let input = "<pre><code class=\"language-js\">let <mark>a</mark></code></pre>";
    let options = json!({ "theme": "github-light", "skipElements": true });
    assert_eq!(highlight(input, options)?, input);
    Ok(())
}
