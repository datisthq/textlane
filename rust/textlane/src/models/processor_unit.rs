use super::Processor;
use textlane_core::Error;

// Linear-time guards: a quadratic regression at these sizes takes minutes, so a loose debug
// limit still catches one while tests run in parallel on a loaded machine
const LIMIT_SECS: u64 = if cfg!(debug_assertions) { 20 } else { 5 };

fn render(input: &str) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::ToHtml::default())
        .with(textlane_html::Stringify::default())
        .freeze()?;
    Ok(processor.process(input)?.value)
}

#[test]
fn processes_markdown_into_html() -> Result<(), Error> {
    assert_eq!(render("# a *b*\n")?, "<h1>a <em>b</em></h1>");
    Ok(())
}

#[test]
fn survives_deeply_nested_emphasis() -> Result<(), Error> {
    let depth = 100_000;
    let input = format!("{}a{}", "*".repeat(depth), "*".repeat(depth));
    let output = render(&input)?;
    assert!(output.starts_with("<p><strong><strong>"));
    Ok(())
}

#[test]
fn pairs_many_unmatched_runs_in_linear_time() -> Result<(), Error> {
    let input = "a**b ".repeat(50_000) + &"*c".repeat(50_000);
    let started = std::time::Instant::now();
    render(&input)?;
    assert!(
        started.elapsed().as_secs() < LIMIT_SECS,
        "took {:?}",
        started.elapsed()
    );
    Ok(())
}

#[test]
fn rejects_many_unclosed_code_spans_in_linear_time() -> Result<(), Error> {
    let input: String = (1..=1_000).map(|size| "`".repeat(size) + "a").collect();
    let started = std::time::Instant::now();
    render(&input)?;
    assert!(
        started.elapsed().as_secs() < LIMIT_SECS,
        "took {:?}",
        started.elapsed()
    );
    Ok(())
}

#[test]
fn survives_deeply_nested_containers() -> Result<(), Error> {
    for prefix in ["- ", "> ", "1. "] {
        let input = prefix.repeat(20_000) + "a\n";
        let started = std::time::Instant::now();
        render(&input)?;
        assert!(
            started.elapsed().as_secs() < LIMIT_SECS,
            "{prefix:?} took {:?}",
            started.elapsed()
        );
    }
    Ok(())
}

#[test]
fn continues_deep_containers_over_blank_lines_in_linear_time() -> Result<(), Error> {
    let items = (0..20_000).map(|_| "- a\n").collect::<String>();
    for input in [
        "- ".repeat(20_000) + "a\n" + &"\n".repeat(20_000),
        "- ".repeat(20_000) + "a\n" + &"  \n".repeat(10_000),
        items,
    ] {
        let started = std::time::Instant::now();
        render(&input)?;
        assert!(
            started.elapsed().as_secs() < LIMIT_SECS,
            "took {:?}",
            started.elapsed()
        );
    }
    Ok(())
}

#[test]
fn resolves_many_setext_underlines_in_linear_time() -> Result<(), Error> {
    for input in [
        "a\n".to_string() + &"=\n".repeat(50_000),
        "a\n-\n".repeat(25_000),
    ] {
        let started = std::time::Instant::now();
        render(&input)?;
        assert!(
            started.elapsed().as_secs() < LIMIT_SECS,
            "took {:?}",
            started.elapsed()
        );
    }
    Ok(())
}

#[test]
fn resolves_hostile_links_in_linear_time() -> Result<(), Error> {
    let definition = "[a]: b\n\n";
    for input in [
        definition.to_string() + &"[".repeat(20_000) + &"]".repeat(20_000),
        definition.to_string() + &"[ ".repeat(10_000) + &" ]".repeat(10_000),
        "[".repeat(20_000) + &"[a](b)".repeat(20_000),
        "[a](b 'c\n".repeat(10_000),
        "[a](b".repeat(20_000),
        "![".repeat(10_000) + &"](b)".repeat(10_000),
        definition.to_string() + &"[a][a]".repeat(20_000),
        "a\\*".repeat(30_000),
    ] {
        let started = std::time::Instant::now();
        render(&input)?;
        let took = started.elapsed();
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..20]
        );
    }
    Ok(())
}

fn render_gfm(input: &str) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::Gfm::default())
        .with(textlane_md::ToHtml::default())
        .with(textlane_html::Stringify::default())
        .freeze()?;
    Ok(processor.process(input)?.value)
}

fn format_gfm(input: &str) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::Gfm::default())
        .with(textlane_md::Stringify::default())
        .freeze()?;
    Ok(processor.process(input)?.value)
}

#[test]
fn resolves_hostile_footnotes_in_linear_time() -> Result<(), Error> {
    let definitions: String = (0..10_000)
        .map(|index| format!("[^{index}]: a\n"))
        .collect();
    let calls: String = (0..10_000).map(|index| format!("[^{index}]")).collect();
    let definition = "\n\n[^a]: b\n";
    for input in [
        calls.clone() + "\n\n" + &definitions,
        "[^a]: ".repeat(20_000) + "[^a]\n",
        "[^a]".to_string() + &"\n\n[^a]: ".repeat(20_000),
        "[^".repeat(30_000) + definition,
        "[^a".repeat(30_000) + definition,
        "![^a".repeat(30_000) + &"]".repeat(30_000) + definition,
        "![^".to_string() + &"a".repeat(100_000) + &"]".repeat(30_000) + definition,
        "![^a] ".repeat(30_000) + definition,
        "[^b]".repeat(30_000) + definition,
        "[^a]".repeat(30_000) + definition,
    ] {
        let started = std::time::Instant::now();
        render_gfm(&input)?;
        format_gfm(&input)?;
        let took = started.elapsed();
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..12]
        );
    }
    Ok(())
}

fn render_def_list(input: &str) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::Gfm::default())
        .with(textlane_md::DefinitionList)
        .with(textlane_md::ToHtml::default())
        .with(textlane_html::Stringify::default())
        .freeze()?;
    Ok(processor.process(input)?.value)
}

fn format_def_list(input: &str) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::DefinitionList)
        .with(textlane_md::Stringify::default())
        .freeze()?;
    Ok(processor.process(input)?.value)
}

#[test]
fn renders_definition_lists() -> Result<(), Error> {
    let html = "<dl>\n<dt>a</dt>\n<dd>b\n</dd>\n</dl>";
    assert_eq!(render_def_list("a\n: b")?, html);
    assert_eq!(format_def_list("a\n: b")?, "a\n:   b\n");
    Ok(())
}

#[test]
fn resolves_hostile_definition_lists_in_linear_time() -> Result<(), Error> {
    for input in [
        "a\n: b\n".repeat(10_000),
        "a\n".repeat(20_000) + ": b\n",
        "[x]: y\n".repeat(10_000) + &": b\n".repeat(10_000),
        "[x]: y\n: b\n\n".repeat(10_000),
        "# h\n: b\n".repeat(10_000),
        "> a\n: b\n".repeat(10_000),
        "> a\n".to_string() + &"b\n".repeat(20_000) + ": c\n",
        "a\n".to_string() + &"===\n".repeat(20_000) + ": b\n",
        "a\n:\n\n".repeat(10_000),
        "| a |\n| - |\n".to_string() + &"b\n: c\n".repeat(10_000),
    ] {
        let started = std::time::Instant::now();
        render_def_list(&input)?;
        format_def_list(&input)?;
        let took = started.elapsed();
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..12]
        );
    }
    Ok(())
}

#[test]
fn rejects_hostile_html_in_linear_time() -> Result<(), Error> {
    for input in [
        "a <!-- b ".repeat(20_000),
        "a <? b ".repeat(20_000),
        "a <!B c ".repeat(20_000),
        "a <![CDATA[ b ".repeat(20_000),
        "a <b c=\" ".repeat(20_000),
        "a <b c='\n".repeat(20_000),
        "<div>\n".repeat(20_000),
        "<!--\n".repeat(20_000),
    ] {
        let started = std::time::Instant::now();
        render(&input)?;
        let took = started.elapsed();
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..12]
        );
    }
    Ok(())
}

fn format_markdown(input: &str) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::Stringify::default())
        .freeze()?;
    Ok(processor.process(input)?.value)
}

#[test]
fn formats_markdown() -> Result<(), Error> {
    assert_eq!(format_markdown("a *b*\n\n- c\n- d")?, "a *b*\n\n* c\n* d\n");
    Ok(())
}

#[test]
fn formats_deep_nesting_in_linear_time() -> Result<(), Error> {
    for input in [
        "*".repeat(100_000) + "a" + &"*".repeat(100_000),
        "> ".repeat(20_000) + "a\n",
        "- ".repeat(20_000) + "a\n",
        "1. ".repeat(20_000) + "a\n",
        "[a](b ".repeat(10_000) + "*c*" + &")".repeat(10_000),
    ] {
        let started = std::time::Instant::now();
        format_markdown(&input)?;
        let took = started.elapsed();
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..12]
        );
    }
    Ok(())
}

#[test]
fn formats_hostile_phrasing_in_linear_time() -> Result<(), Error> {
    for input in [
        "*a**b*".repeat(5_000),
        "_a__b_".repeat(5_000),
        "*a*_b_".repeat(5_000),
        "**a*b*c**".repeat(5_000),
        "a_b".repeat(30_000),
        "a\\*".repeat(30_000),
        "`".to_string() + &"\n# a".repeat(30_000) + "`",
        "a  \n".repeat(30_000),
        "<a>\nb\n".repeat(20_000),
    ] {
        let started = std::time::Instant::now();
        format_markdown(&input)?;
        let took = started.elapsed();
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..12]
        );
    }
    Ok(())
}

fn reformat_html(input: &str) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_html::Parse::default())
        .with(textlane_html::Stringify::default())
        .freeze()?;
    Ok(processor.process(input)?.value)
}

#[test]
fn parses_html() -> Result<(), Error> {
    assert_eq!(
        reformat_html("<p>a<b>c")?,
        "<html><head></head><body><p>a<b>c</b></p></body></html>"
    );
    Ok(())
}

#[test]
fn parses_hostile_html_in_linear_time() -> Result<(), Error> {
    let numbered = |template: &str| -> String {
        (0..20_000)
            .map(|index| template.replace('N', &index.to_string()))
            .collect()
    };
    for input in [
        "<div>".repeat(100_000),
        "<svg>".repeat(100_000),
        "<template>".repeat(50_000),
        "<template><div>".repeat(50_000),
        "<template><table>".repeat(50_000),
        numbered("<b x=N>"),
        "<a><b>".repeat(20_000) + &"</a>".repeat(20_000),
        "<table>".to_string() + &"x<td>".repeat(20_000),
        "<table>".to_string() + &"<p>x".repeat(20_000),
        "<div ".to_string() + &numbered("aN=1 ") + ">",
        "<li><ul>".repeat(20_000),
        "<div>".repeat(20_000) + &"x<i></i>".repeat(20_000),
        "<font color=red>".repeat(20_000) + &"<p>x".repeat(20_000),
        "&amp".repeat(50_000),
    ] {
        let started = std::time::Instant::now();
        reformat_html(&input)?;
        let took = started.elapsed();
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..12]
        );
    }
    Ok(())
}

#[test]
fn sanitizes_and_slugs_hostile_html_in_linear_time() -> Result<(), Error> {
    let processor = Processor::new()
        .with(textlane_html::Parse::default())
        .with(textlane_html::Sanitize::default())
        .with(textlane_html::Slug::default())
        .with(textlane_html::Stringify::default())
        .freeze()?;
    for input in [
        "<div>".repeat(100_000),
        "<unknown>".repeat(100_000) + "x",
        "<table><tr><td>".repeat(20_000),
        "<h1>a</h1>".repeat(20_000),
        "<h1>".to_string() + &"<b>a".repeat(50_000),
        "<h1><b>a".repeat(20_000),
        "<h1>".to_string() + &"a".repeat(1_000_000),
    ] {
        let started = std::time::Instant::now();
        processor.process(&input)?;
        let took = started.elapsed();
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..12]
        );
    }
    Ok(())
}

#[test]
fn converts_hostile_html_to_markdown_in_linear_time() -> Result<(), Error> {
    let processor = Processor::new()
        .with(textlane_html::Parse::default())
        .with(textlane_html::ToMarkdown::default())
        .freeze()?;
    let datalist =
        "<datalist id=d>".to_string() + &"<option>x".repeat(20_000) + "</datalist>";
    for input in [
        "<div>".repeat(100_000) + "x",
        "<span>".repeat(100_000) + " x ",
        "<b> </b>".repeat(100_000),
        "<span>a</span>".repeat(100_000),
        "<span>a".repeat(20_000),
        "<a href=x><div>".repeat(20_000) + "x",
        "<del><div>".repeat(20_000) + "x",
        "<del>".repeat(20_000) + "<p>x",
        "<a href=x><del>".repeat(10_000) + "<p>x",
        "<table>".to_string() + &"<tr><td>x".repeat(20_000),
        "<table>".to_string() + &"<tr><td colspan=3 rowspan=3>x".repeat(10_000),
        "<table><tr>".to_string() + &"<td colspan=2>x".repeat(10_000),
        "<table><tr>".to_string() + &"<td colspan=2>x".repeat(100_000),
        "<table>".to_string() + &"<tr><td rowspan=1000 colspan=1000>x".repeat(10_000),
        "<table>".to_string() + &"<tr><td rowspan=65534>x".repeat(20_000),
        "<table>".to_string() + &"<td>".repeat(50_000) + &"<tr>".repeat(50_000),
        "<table><tr><td>".repeat(20_000),
        "<ul>".to_string() + &"<li>x".repeat(50_000),
        "<li><ul>".repeat(20_000),
        "<li><p>".repeat(20_000),
        "<dl>".to_string() + &"<dt>a<dd>b".repeat(20_000),
        "<dl><dd>".repeat(20_000),
        "<blockquote>".repeat(50_000) + "x",
        "<q>".repeat(20_000) + "x",
        "<q>a".repeat(20_000),
        "<video>".repeat(20_000) + "x",
        "<video poster=p>".repeat(20_000) + "x",
        "<select>".to_string() + &"<option>x".repeat(50_000),
        datalist + &"<input list=d>".repeat(20_000),
        "<pre>".to_string() + &"a\n".repeat(100_000),
        "<code>".repeat(20_000) + "x",
    ] {
        let started = std::time::Instant::now();
        let alloc = textlane_core::Allocator::new();
        let mut file = textlane_core::File::new();
        let tree = processor.parse(&input, &mut file, &alloc)?;
        processor.run(tree, &mut file, &alloc)?;
        let took = started.elapsed();
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..15]
        );
    }
    Ok(())
}

#[test]
fn handles_hostile_gfm_tables_in_linear_time() -> Result<(), Error> {
    let gfm = || textlane_md::Gfm::default();
    let to_html = Processor::new()
        .with(textlane_md::Parse::default())
        .with(gfm())
        .with(textlane_md::ToHtml::default())
        .with(textlane_html::Stringify::default())
        .freeze()?;
    let to_markdown = Processor::new()
        .with(textlane_md::Parse::default())
        .with(gfm())
        .with(textlane_md::Stringify::default())
        .freeze()?;
    let head = "|a".repeat(10_000) + "|\n" + &"|-".repeat(10_000) + "|\n";
    for input in [
        head.clone() + &"|b".repeat(10_000) + "|\n",
        head.clone() + &"|b".repeat(50_000),
        "| a | b |\n| - | - |\n".to_string() + &"| c | d |\n".repeat(30_000),
        "| a |\n| - |\n".to_string() + &"| `b\\|c` \\| d |\n".repeat(20_000),
        "a | b\n".repeat(30_000),
        "| a |\n|-x\n".repeat(30_000),
        "| a |\n".to_string() + &"| - ".repeat(30_000) + "|",
        "> | a |\n".repeat(30_000),
        "- | a |\n  | - |\n".repeat(10_000),
        "|".repeat(100_000) + "\n|-|",
    ] {
        for processor in [&to_html, &to_markdown] {
            let started = std::time::Instant::now();
            processor.process(&input)?;
            let took = started.elapsed();
            assert!(
                took.as_secs() < LIMIT_SECS,
                "{:?}… took {took:?}",
                &input[..12]
            );
        }
    }
    Ok(())
}

#[test]
fn handles_hostile_gfm_in_linear_time() -> Result<(), Error> {
    let render = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::Gfm::default())
        .with(textlane_md::ToHtml::default())
        .with(textlane_html::Stringify::default())
        .freeze()?;
    let format = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::Gfm::default())
        .with(textlane_md::Stringify::default())
        .freeze()?;
    for input in [
        "x_www.a_".repeat(20_000),
        "www.".repeat(50_000) + "_",
        "-www.a.b ".repeat(20_000),
        "http://a_".repeat(20_000),
        "www.a.com/".to_string() + &"!".repeat(100_000) + "x",
        "www.a.com/".to_string() + &")".repeat(100_000) + "x",
        "www.a.com/".to_string() + &"]".repeat(100_000) + "x",
        "www.a.com/".to_string() + &"&a".repeat(50_000) + "x",
        "www.a.com/".to_string() + &"(".repeat(50_000) + &")".repeat(50_000),
        "a.".repeat(50_000),
        "a@".repeat(50_000),
        "a@b.".repeat(30_000),
        "a.-+_".repeat(30_000) + "@b.c_",
        "~".repeat(100_000),
        "~a ".repeat(30_000) + &"a~ ".repeat(30_000),
        "~~a ~b ".repeat(20_000) + &"c~ d~~ ".repeat(20_000),
        "~~*".repeat(20_000) + "a" + &"*~~".repeat(20_000),
        "[".repeat(20_000) + &"www.a.com ".repeat(20_000),
        "- [ ] a\n".repeat(20_000),
        "- ".to_string() + &"[a]: b\n  ".repeat(20_000) + "[x] c",
        "- [x] a\n\n  b\n".repeat(20_000),
        "[ ] ".repeat(30_000),
    ] {
        for processor in [&render, &format] {
            let started = std::time::Instant::now();
            processor.process(&input)?;
            let took = started.elapsed();
            assert!(
                took.as_secs() < LIMIT_SECS,
                "{:?}… took {took:?}",
                &input[..12]
            );
        }
    }
    Ok(())
}

#[test]
fn handles_hostile_math_in_linear_time() -> Result<(), Error> {
    let math = || textlane_md::Math::default();
    let render = Processor::new()
        .with(textlane_md::Parse::default())
        .with(math())
        .with(textlane_md::ToHtml::default())
        .with(textlane_html::Stringify::default())
        .freeze()?;
    let format = Processor::new()
        .with(textlane_md::Parse::default())
        .with(math())
        .with(textlane_md::Stringify::default())
        .freeze()?;
    for input in [
        (1..=1_000).map(|size| "$".repeat(size) + "a").collect(),
        "$".repeat(100_000),
        "$a ".repeat(50_000),
        "\\$$".repeat(30_000),
        "$$\n".repeat(30_000),
        "$$ $".repeat(30_000),
    ] {
        for processor in [&render, &format] {
            let started = std::time::Instant::now();
            processor.process(&input)?;
            let took = started.elapsed();
            assert!(
                took.as_secs() < LIMIT_SECS,
                "{:?}… took {took:?}",
                &input[..12]
            );
        }
    }
    Ok(())
}

#[test]
fn handles_hostile_mdx_in_linear_time() -> Result<(), Error> {
    let mdx = || textlane_md::Mdx::default();
    let render = Processor::new()
        .with(textlane_md::Parse::default())
        .with(mdx())
        .with(textlane_md::ToHtml::default())
        .freeze()?;
    let format = Processor::new()
        .with(textlane_md::Parse::default())
        .with(mdx())
        .with(textlane_md::Stringify::default())
        .freeze()?;
    let inputs: Vec<(String, bool)> = vec![
        // Indenting nested flow elements grows the output with depth squared, by nature
        ("<a>\n".repeat(20_000) + &"</a>\n".repeat(20_000), false),
        (
            "x ".to_string() + &"<a>".repeat(50_000) + &"</a>".repeat(50_000),
            true,
        ),
        ("<a ".to_string() + &"b ".repeat(100_000), true),
        ("<a b=\"".to_string() + &"c\n".repeat(50_000), true),
        ("<a/>".repeat(50_000), true),
        ("a <b {...c} d={e} />".repeat(20_000), true),
        ("{".repeat(100_000), true),
        ("{}".repeat(50_000), true),
        ("{".repeat(50_000) + &"}".repeat(50_000), true),
        ("{(".to_string() + &"}".repeat(100_000), true),
        ("<a {(".to_string() + &"}".repeat(100_000), true),
        (
            "{[".to_string() + &"(".repeat(100_000) + &")".repeat(100_000) + "]}",
            true,
        ),
        ("x {a\n".to_string() + &"b\n".repeat(50_000) + "}", true),
        ("import a from 'b'\n".repeat(20_000), true),
        (
            "export const a = {\n".to_string() + &"b,\n".repeat(50_000) + "}\n",
            true,
        ),
    ];
    for (input, formats) in &inputs {
        let processors: &[&_] = if *formats {
            &[&render, &format]
        } else {
            &[&render]
        };
        for processor in processors {
            let started = std::time::Instant::now();
            let _ = processor.process(input);
            let took = started.elapsed();
            assert!(
                took.as_secs() < LIMIT_SECS,
                "{:?}… took {took:?}",
                &input[..12]
            );
        }
    }
    Ok(())
}

#[test]
fn checks_setext_underlines_in_linear_time() -> Result<(), Error> {
    for input in [
        "a\n".to_string() + &"=x\n".repeat(50_000),
        "[a]: b\n".repeat(50_000) + "===\n",
        "[a]: b\n===\n".repeat(30_000),
        "[a]: b\n---\n".repeat(30_000),
        "a\n".to_string() + &"-- -\n".repeat(50_000),
    ] {
        let started = std::time::Instant::now();
        render(&input)?;
        assert!(
            started.elapsed().as_secs() < LIMIT_SECS,
            "took {:?}",
            started.elapsed()
        );
    }
    Ok(())
}

#[test]
fn reparses_hostile_raw_html_in_linear_time() -> Result<(), Error> {
    let markdown = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::ToHtml {
            allow_dangerous_html: true,
            ..Default::default()
        })
        .with(textlane_html::Raw::default())
        .with(textlane_html::Stringify::default())
        .freeze()?;
    for input in [
        "<div>\n\n".repeat(50_000),
        "<span>".repeat(100_000),
        "a <i>b</i> ".repeat(50_000),
        "<i>a".repeat(20_000),
        "<table>\n\n".to_string() + &"<td>x\n\n".repeat(20_000),
        "<title>\n\n".to_string() + &"a <b>c</b> ".repeat(50_000),
        "x &amp".repeat(50_000),
        "<!--\n\n".repeat(50_000),
    ] {
        let started = std::time::Instant::now();
        markdown.process(&input)?;
        let took = started.elapsed();
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..12]
        );
    }
    let html = Processor::new()
        .with(textlane_html::Parse::default())
        .with(textlane_html::Raw::default())
        .with(textlane_html::Stringify::default())
        .freeze()?;
    for input in [
        "<div>".repeat(100_000),
        "<svg>".repeat(100_000),
        "<a><b>".repeat(20_000) + &"</a>".repeat(20_000),
        "<template>".repeat(50_000),
    ] {
        let started = std::time::Instant::now();
        html.process(&input)?;
        let took = started.elapsed();
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..12]
        );
    }
    Ok(())
}

#[test]
fn formats_hostile_html_in_linear_time() -> Result<(), Error> {
    let indented = Processor::new()
        .with(textlane_html::Parse::default())
        .with(textlane_html::Format::default())
        .with(textlane_html::Stringify::default())
        .freeze()?;
    for input in [
        "<p>a</p>\n".repeat(100_000),
        "<span>a\n</span>".repeat(100_000),
        "<div>".to_string() + &"a\n".repeat(100_000),
        "<head>".to_string() + &"<b>x</b>\n".repeat(50_000),
        "<div>".repeat(100_000),
        "<div>\n".repeat(100_000),
        "<span>".repeat(100_000) + "x",
        "<svg>".repeat(100_000),
    ] {
        let started = std::time::Instant::now();
        indented.process(&input)?;
        let took = started.elapsed();
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..12]
        );
    }
    Ok(())
}

fn reparse_natural_language(
    input: &str,
    language: textlane_nl::Language,
) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_nl::Parse { language })
        .with(textlane_nl::Stringify)
        .freeze()?;
    Ok(processor.process(input)?.value)
}

#[test]
fn parses_hostile_natural_language_in_linear_time() -> Result<(), Error> {
    let inputs = [
        ".".repeat(1_000_000),
        "!?".repeat(200_000),
        "a".repeat(1_000_000),
        "a-".repeat(200_000),
        "a.".repeat(200_000),
        "a/".repeat(200_000),
        "&a ".repeat(100_000),
        "a. ".repeat(100_000),
        "A. ".repeat(100_000),
        ". ".repeat(100_000),
        "Mr. ".repeat(100_000),
        "e.g. ".repeat(100_000),
        "A. , ".repeat(100_000),
        "a.\" ".repeat(100_000),
        "a\n\n".repeat(100_000),
        " \n".repeat(100_000),
        "'tis 'n' ol' w/ ".repeat(50_000),
        "😀a".repeat(100_000),
    ];
    for input in inputs {
        for language in [textlane_nl::Language::Latin, textlane_nl::Language::English] {
            let started = std::time::Instant::now();
            assert_eq!(reparse_natural_language(&input, language)?, input);
            assert!(
                started.elapsed().as_secs() < LIMIT_SECS,
                "{:?}… took {:?}",
                input.chars().take(8).collect::<String>(),
                started.elapsed()
            );
        }
    }
    Ok(())
}

// What a bridge into nlcst does with the parser it reads from the slots
struct NaturalLanguageBridge;

struct NaturalLanguageBridgeParser(
    std::sync::Arc<dyn textlane_core::NaturalLanguageParser>,
);

impl textlane_core::Plugin for NaturalLanguageBridge {
    fn attach(&self, context: &mut textlane_core::Attach) -> Result<(), Error> {
        context.set_parser(|slots| {
            let parser = slots
                .get::<textlane_core::NaturalLanguageParserSlot>()
                .and_then(|slot| slot.parser.clone())
                .ok_or_else(|| {
                    Error::new("missing-parser", "no natural-language parser")
                })?;
            Ok(Box::new(NaturalLanguageBridgeParser(parser)))
        });
        Ok(())
    }
}

impl textlane_core::Parser for NaturalLanguageBridgeParser {
    fn output(&self) -> textlane_core::TreeKind {
        textlane_core::TreeKind::Nlcst
    }

    fn parse<'a>(
        &self,
        input: &'a str,
        _file: &mut textlane_core::File,
        alloc: &'a textlane_core::Allocator,
    ) -> Result<textlane_core::Tree<'a>, Error> {
        let content = self.0.tokenize(input, 0, alloc);
        let paragraph = self.0.tokenize_paragraph(content, None, alloc);
        Ok(textlane_core::Tree::Nlcst(
            self.0.tokenize_root(paragraph, None, alloc),
        ))
    }
}

#[test]
fn hands_the_natural_language_parser_to_plugins() -> Result<(), Error> {
    let processor = Processor::new()
        .with(NaturalLanguageBridge)
        .with(textlane_nl::Stringify)
        .freeze()?;
    let alloc = textlane_core::Allocator::new();
    let mut file = textlane_core::File::new();
    let input = "Mr. Smith left. He said: e.g. this.\n\nNext one.";
    let tree = processor.parse(input, &mut file, &alloc)?;
    let expected =
        textlane_nl::parse_natural_language(input, &Default::default(), &alloc);
    assert_eq!(tree, textlane_core::Tree::Nlcst(expected));
    assert_eq!(processor.stringify(&tree, &mut file)?, input);
    Ok(())
}

#[test]
fn checks_hostile_natural_language_in_linear_time() -> Result<(), Error> {
    let processor = Processor::new()
        .with(textlane_nl::Parse::default())
        .with(textlane_nl::Equality::default())
        .with(textlane_nl::Equality {
            binary: Some(true),
            ..Default::default()
        })
        .with(textlane_nl::Readability::default())
        .with(textlane_nl::Passive::default())
        .with(textlane_nl::RepeatedWords)
        .with(textlane_nl::SentenceSpacing::default())
        .with(textlane_nl::Stringify)
        .freeze()?;
    for input in [
        "he or she and his/her ".repeat(10_000),
        "the chairman was fed by a cripple, ".repeat(5_000),
        "fed ".repeat(50_000),
        "a ,.;:-—(\"' ".repeat(10_000),
        "Bad.  Worse.\n\n".repeat(10_000),
        "antidisestablishmentarianism ".repeat(10_000),
        "a".repeat(1_000_000),
        "(".repeat(100_000) + "cripple" + &")".repeat(100_000),
    ] {
        let started = std::time::Instant::now();
        let output = processor.process(&input)?;
        let took = started.elapsed();
        assert_eq!(output.value, input);
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..12]
        );
    }
    Ok(())
}

#[test]
fn checks_hostile_natural_language_typography_in_linear_time() -> Result<(), Error> {
    let plugins = [
        ("nl.smartypants", serde_json::json!(null)),
        (
            "nl.smartypants",
            serde_json::json!({ "ellipses": "spaced", "dashes": "inverted" }),
        ),
        (
            "nl.smartypants",
            serde_json::json!({ "quotes": false, "backticks": "all", "dashes": "oldschool" }),
        ),
        ("nl.quotes", serde_json::json!(null)),
        ("nl.quotes", serde_json::json!({ "preferred": "straight" })),
        ("nl.contractions", serde_json::json!(null)),
        ("nl.indefiniteArticle", serde_json::json!(null)),
    ];
    let inputs = [
        "\"".repeat(125_000),
        "'".repeat(125_000),
        "\" ".repeat(50_000),
        "'a' ".repeat(25_000),
        "“‘«".repeat(25_000),
        "-- ".repeat(50_000),
        "—-".repeat(50_000),
        "–-".repeat(50_000),
        ". ".repeat(75_000),
        ". . . a ".repeat(25_000),
        "...".repeat(50_000),
        "``''".repeat(50_000),
        "é dont ".repeat(25_000),
        "\"dont\" - ".repeat(12_500),
        "A apple ".repeat(25_000),
        "é an 1000 ".repeat(25_000),
    ];
    for input in &inputs {
        for (name, options) in &plugins {
            let build = crate::actions::plugin::find::find_plugin(name)
                .ok_or_else(|| Error::new("unknown-plugin", *name))?;
            let processor = Processor::new()
                .with(textlane_nl::Parse::default())
                .with_boxed(build(name, options.clone())?)
                .with(textlane_nl::Stringify)
                .freeze()?;
            let started = std::time::Instant::now();
            processor.process(input)?;
            let took = started.elapsed();
            assert!(
                took.as_secs() < LIMIT_SECS,
                "{:?}… took {took:?}",
                input.chars().take(8).collect::<String>()
            );
        }
    }
    Ok(())
}

#[test]
fn places_many_messages_on_non_ascii_natural_language_in_linear_time() -> Result<(), Error>
{
    let processor = Processor::new()
        .with(textlane_nl::Parse::default())
        .with(textlane_nl::Equality::default())
        .with(textlane_nl::RepeatedWords)
        .with(textlane_nl::Stringify)
        .freeze()?;
    for input in ["é he ".repeat(50_000), "é he he ".repeat(25_000)] {
        let started = std::time::Instant::now();
        let output = processor.process(&input)?;
        let took = started.elapsed();
        assert!(output.file.messages.len() >= 25_000);
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..12]
        );
    }
    Ok(())
}

fn markdown_to_natural_language(input: &str) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::ToNaturalLanguage::default())
        .with(textlane_nl::Stringify)
        .freeze()?;
    Ok(processor.process(input)?.value)
}

fn html_to_natural_language(input: &str) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_html::Parse::default())
        .with(textlane_html::ToNaturalLanguage)
        .with(textlane_nl::Stringify)
        .freeze()?;
    Ok(processor.process(input)?.value)
}

#[test]
fn bridges_markdown_and_html_into_natural_language() -> Result<(), Error> {
    assert_eq!(
        markdown_to_natural_language("# A *b*.\n\nC `d`.\n")?,
        "A b.\n\nC `d`."
    );
    assert_eq!(
        html_to_natural_language("<p>A <b>b</b>.</p><div>C<br>d <code>e</code></div>")?,
        "A b.C\nd e"
    );
    Ok(())
}

#[test]
fn bridges_hostile_markdown_into_natural_language_in_linear_time() -> Result<(), Error> {
    for input in [
        "> ".repeat(20_000) + "a\n",
        "- ".repeat(20_000) + "a\n",
        "*".repeat(50_000) + "a" + &"*".repeat(50_000),
        "[".repeat(50_000) + "a" + &"](b)".repeat(50_000),
        "Mr. a b. ".repeat(50_000),
        "a\n".repeat(30_000),
        "a\n\n".repeat(30_000),
        "`a` ![b](c) d\\\n".repeat(5_000),
        "> é😀 b\n".repeat(50_000),
    ] {
        let started = std::time::Instant::now();
        markdown_to_natural_language(&input)?;
        let took = started.elapsed();
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..12]
        );
    }
    Ok(())
}

#[test]
fn bridges_hostile_html_into_natural_language_in_linear_time() -> Result<(), Error> {
    for input in [
        "<div>".repeat(100_000) + "a",
        "<p>".to_string() + &"<span>".repeat(100_000) + "a",
        "<div>".to_string() + &"<ins>".repeat(100_000) + "a",
        "<div>".to_string() + &"<code>".repeat(50_000) + "a",
        "<blockquote>".repeat(50_000) + "a",
        "<p>".to_string() + &"Mr. a b. ".repeat(50_000),
        "<div>".to_string() + &"a<br>".repeat(100_000),
        "<div>".to_string() + &"a<p>b</p>".repeat(50_000),
        "<p>".to_string() + &"&amp;é😀 ".repeat(50_000),
    ] {
        let started = std::time::Instant::now();
        html_to_natural_language(&input)?;
        let took = started.elapsed();
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..12]
        );
    }
    Ok(())
}

fn compile_mdx(input: &str, options: textlane_mdx::Compile) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::Mdx::default())
        .with(textlane_md::ToHtml::default())
        .with(options)
        .freeze()?;
    Ok(processor.process(input)?.value)
}

#[test]
fn compiles_deeply_nested_markdown_into_javascript() -> Result<(), Error> {
    let jsx = textlane_mdx::Compile {
        jsx: true,
        ..Default::default()
    };
    let started = std::time::Instant::now();
    let output = compile_mdx(&format!("{}a", "> ".repeat(100_000)), jsx)?;
    assert!(output.contains("<_components.blockquote>{\"\\n\"}<_components.blockquote>"));
    // Calls indent each level's object, so the output itself grows with depth squared
    let output = compile_mdx(&format!("{}a", "> ".repeat(2_000)), Default::default())?;
    assert!(output.contains("_jsxs(_components.blockquote, {"));
    assert!(
        started.elapsed().as_secs() < LIMIT_SECS,
        "took {:?}",
        started.elapsed()
    );
    Ok(())
}

#[test]
fn compiles_long_markdown_into_javascript_in_linear_time() -> Result<(), Error> {
    let input = "a *b* [c](d) `e`\n\n".repeat(20_000);
    let started = std::time::Instant::now();
    let output = compile_mdx(&input, Default::default())?;
    assert!(output.ends_with("}\n"));
    assert!(
        started.elapsed().as_secs() < LIMIT_SECS,
        "took {:?}",
        started.elapsed()
    );
    Ok(())
}

#[test]
fn compiles_deeply_nested_jsx_into_javascript() -> Result<(), Error> {
    let depth = 10_000;
    let input = format!("{}b{}", "<a>".repeat(depth), "</a>".repeat(depth));
    let started = std::time::Instant::now();
    let jsx = textlane_mdx::Compile {
        jsx: true,
        ..Default::default()
    };
    let output = compile_mdx(&input, jsx)?;
    assert!(output.contains("<a><a>"));
    // JSX in an expression nests only as deep as oxc may parse it
    let inner = format!("{}b{}", "<a>".repeat(1_000), "</a>".repeat(1_000));
    let output = compile_mdx(&format!("{{{inner}}}"), Default::default())?;
    assert!(output.contains("_jsx(\"a\", {"));
    assert!(
        started.elapsed().as_secs() < LIMIT_SECS,
        "took {:?}",
        started.elapsed()
    );
    Ok(())
}

#[test]
fn compiles_many_components_and_exports_in_linear_time() -> Result<(), Error> {
    let mut input = String::new();
    for index in 0..10_000 {
        input.push_str(&format!(
            "export const a{index} = {index}\n\n<X{index} a={{a{index}}} />\n\n"
        ));
    }
    let started = std::time::Instant::now();
    let output = compile_mdx(&input, Default::default())?;
    assert!(output.contains("if (!X9999) _missingMdxReference(\"X9999\", true);"));
    let body = textlane_mdx::Compile {
        output_format: Some("function-body".to_string()),
        ..Default::default()
    };
    let output = compile_mdx(&input, body)?;
    assert!(output.contains("a9999,"));
    assert!(
        started.elapsed().as_secs() < LIMIT_SECS,
        "took {:?}",
        started.elapsed()
    );
    Ok(())
}

#[test]
fn parses_deeply_recursive_mdx_scripts_without_overflow() -> Result<(), Error> {
    let compile = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::Mdx::default())
        .with(textlane_md::ToHtml::default())
        .with(textlane_mdx::Compile::default())
        .freeze()?;
    let depth = 5_000;
    for input in [
        format!("{{{}{}}}", "<a>".repeat(depth), "</a>".repeat(depth)),
        format!("{{{}a}}", "!".repeat(depth)),
        format!("{{{}a}}", "a=".repeat(depth)),
        format!("{{{}a{}}}", "(".repeat(depth), ")".repeat(depth)),
        format!("{{{}x}}", "x=>".repeat(depth)),
        format!("{{{}a}}", "new ".repeat(depth)),
        format!("{{{}c}}", "a?b:".repeat(depth)),
        format!(
            "export const a = {}1{}\n",
            "[".repeat(depth),
            "]".repeat(depth)
        ),
    ] {
        let started = std::time::Instant::now();
        // An error is fine; overflowing the stack is not
        let _ = compile.process(&input);
        let took = started.elapsed();
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..12]
        );
    }
    Ok(())
}

#[test]
fn gives_up_on_mdx_scripts_that_keep_failing_in_linear_time() -> Result<(), Error> {
    let render = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::Mdx::default())
        .with(textlane_md::ToHtml::default())
        .freeze()?;
    for input in [
        "{'".to_string() + &"}".repeat(100_000),
        "{/*".to_string() + &" }".repeat(100_000),
        "{`".to_string() + &"a}".repeat(100_000),
        "export const a = /*\n".to_string() + &"a\n\n".repeat(50_000),
    ] {
        let started = std::time::Instant::now();
        let _ = render.process(&input);
        let took = started.elapsed();
        assert!(
            took.as_secs() < LIMIT_SECS,
            "{:?}… took {took:?}",
            &input[..3]
        );
    }
    Ok(())
}

fn render_components(
    input: &str,
    component: textlane_md::Component,
) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::Directive::default())
        .with(textlane_md::Mdx::default())
        .with(component)
        .with(textlane_md::Stringify::default())
        .freeze()?;
    Ok(processor.process(input)?.value)
}

#[test]
fn turns_directives_into_components() -> Result<(), Error> {
    let rule = textlane_md::DirectiveRule {
        name: textlane_md::Names::One("note".into()),
        component: "Callout".into(),
        name_as: Some("type".into()),
        label_as: Some("title".into()),
        ..Default::default()
    };
    let component = textlane_md::Component {
        directives: vec![rule],
        ..Default::default()
    };
    let output = render_components(":::note[Heads up]\nRead *this*.\n:::", component)?;
    let expected =
        "<Callout type=\"note\" title=\"Heads up\">\n  Read *this*.\n</Callout>\n";
    assert_eq!(output, expected);
    Ok(())
}

#[test]
fn groups_many_components_in_linear_time() -> Result<(), Error> {
    let rule = textlane_md::DirectiveRule {
        name: textlane_md::Names::One("card".into()),
        component: "Card".into(),
        group: Some(textlane_md::GroupRule {
            component: "Cards".into(),
            min: None,
            inherit: Vec::new(),
        }),
        ..Default::default()
    };
    let component = textlane_md::Component {
        directives: vec![rule],
        ..Default::default()
    };
    let input = ":::card\na\n:::\n\n".repeat(50_000);
    let started = std::time::Instant::now();
    let output = render_components(&input, component)?;
    assert!(output.starts_with("<Cards>"));
    assert!(started.elapsed().as_secs() < LIMIT_SECS);
    Ok(())
}

#[test]
fn strips_deep_block_quotes_without_recursing() -> Result<(), Error> {
    let input = format!("{}a\n", "> ".repeat(10_000));
    let processor = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::Strip::default())
        .with(textlane_md::Stringify::default())
        .freeze()?;
    assert_eq!(processor.process(&input)?.value, "a\n");
    Ok(())
}

#[test]
fn extracts_many_headings_in_linear_time() -> Result<(), Error> {
    let input = "# a\n\n## b\n\n### c\n\n".repeat(20_000);
    let processor = Processor::new()
        .with(textlane_md::Parse::default())
        .with(textlane_md::HeadingId)
        .with(textlane_md::ToHtml::default())
        .with(textlane_html::UnwrapImages)
        .with(textlane_html::ExtractToc)
        .with(textlane_html::Stringify::default())
        .freeze()?;
    let started = std::time::Instant::now();
    let output = processor.process(&input)?;
    assert_eq!(
        output.file.data["toc"].as_array().map(Vec::len),
        Some(20_000)
    );
    assert!(started.elapsed().as_secs() < LIMIT_SECS);
    Ok(())
}
