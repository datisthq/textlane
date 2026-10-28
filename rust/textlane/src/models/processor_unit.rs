use super::Processor;
use textlane_core::Error;

fn render(input: &str) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_md::parse(Default::default()))
        .with(textlane_md::to_html(Default::default()))
        .with(textlane_html::stringify(Default::default()))
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
        started.elapsed().as_secs() < 5,
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
        started.elapsed().as_secs() < 5,
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
            started.elapsed().as_secs() < 5,
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
            started.elapsed().as_secs() < 5,
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
            started.elapsed().as_secs() < 5,
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
        assert!(took.as_secs() < 5, "{:?}… took {took:?}", &input[..20]);
    }
    Ok(())
}

fn render_gfm(input: &str) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_md::parse(Default::default()))
        .with(textlane_md::gfm(Default::default()))
        .with(textlane_md::to_html(Default::default()))
        .with(textlane_html::stringify(Default::default()))
        .freeze()?;
    Ok(processor.process(input)?.value)
}

fn format_gfm(input: &str) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_md::parse(Default::default()))
        .with(textlane_md::gfm(Default::default()))
        .with(textlane_md::stringify(Default::default()))
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
        assert!(took.as_secs() < 5, "{:?}… took {took:?}", &input[..12]);
    }
    Ok(())
}

fn render_def_list(input: &str) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_md::parse(Default::default()))
        .with(textlane_md::gfm(Default::default()))
        .with(textlane_md::definition_list())
        .with(textlane_md::to_html(Default::default()))
        .with(textlane_html::stringify(Default::default()))
        .freeze()?;
    Ok(processor.process(input)?.value)
}

fn format_def_list(input: &str) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_md::parse(Default::default()))
        .with(textlane_md::definition_list())
        .with(textlane_md::stringify(Default::default()))
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
        assert!(took.as_secs() < 5, "{:?}… took {took:?}", &input[..12]);
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
        assert!(took.as_secs() < 5, "{:?}… took {took:?}", &input[..12]);
    }
    Ok(())
}

fn format_markdown(input: &str) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_md::parse(Default::default()))
        .with(textlane_md::stringify(Default::default()))
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
        assert!(took.as_secs() < 5, "{:?}… took {took:?}", &input[..12]);
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
        assert!(took.as_secs() < 5, "{:?}… took {took:?}", &input[..12]);
    }
    Ok(())
}

fn reformat_html(input: &str) -> Result<String, Error> {
    let processor = Processor::new()
        .with(textlane_html::parse(Default::default()))
        .with(textlane_html::stringify(Default::default()))
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
        assert!(took.as_secs() < 5, "{:?}… took {took:?}", &input[..12]);
    }
    Ok(())
}

#[test]
fn sanitizes_and_slugs_hostile_html_in_linear_time() -> Result<(), Error> {
    let processor = Processor::new()
        .with(textlane_html::parse(Default::default()))
        .with(textlane_html::sanitize(Default::default()))
        .with(textlane_html::slug(Default::default()))
        .with(textlane_html::stringify(Default::default()))
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
        assert!(took.as_secs() < 5, "{:?}… took {took:?}", &input[..12]);
    }
    Ok(())
}

#[test]
fn converts_hostile_html_to_markdown_in_linear_time() -> Result<(), Error> {
    let processor = Processor::new()
        .with(textlane_html::parse(Default::default()))
        .with(textlane_html::to_markdown(Default::default()))
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
        assert!(took.as_secs() < 5, "{:?}… took {took:?}", &input[..15]);
    }
    Ok(())
}

#[test]
fn handles_hostile_gfm_tables_in_linear_time() -> Result<(), Error> {
    let gfm = || textlane_md::gfm(Default::default());
    let to_html = Processor::new()
        .with(textlane_md::parse(Default::default()))
        .with(gfm())
        .with(textlane_md::to_html(Default::default()))
        .with(textlane_html::stringify(Default::default()))
        .freeze()?;
    let to_markdown = Processor::new()
        .with(textlane_md::parse(Default::default()))
        .with(gfm())
        .with(textlane_md::stringify(Default::default()))
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
            assert!(took.as_secs() < 5, "{:?}… took {took:?}", &input[..12]);
        }
    }
    Ok(())
}

#[test]
fn handles_hostile_gfm_in_linear_time() -> Result<(), Error> {
    let render = Processor::new()
        .with(textlane_md::parse(Default::default()))
        .with(textlane_md::gfm(Default::default()))
        .with(textlane_md::to_html(Default::default()))
        .with(textlane_html::stringify(Default::default()))
        .freeze()?;
    let format = Processor::new()
        .with(textlane_md::parse(Default::default()))
        .with(textlane_md::gfm(Default::default()))
        .with(textlane_md::stringify(Default::default()))
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
            assert!(took.as_secs() < 5, "{:?}… took {took:?}", &input[..12]);
        }
    }
    Ok(())
}

#[test]
fn handles_hostile_math_in_linear_time() -> Result<(), Error> {
    let math = || textlane_md::math(Default::default());
    let render = Processor::new()
        .with(textlane_md::parse(Default::default()))
        .with(math())
        .with(textlane_md::to_html(Default::default()))
        .with(textlane_html::stringify(Default::default()))
        .freeze()?;
    let format = Processor::new()
        .with(textlane_md::parse(Default::default()))
        .with(math())
        .with(textlane_md::stringify(Default::default()))
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
            assert!(took.as_secs() < 5, "{:?}… took {took:?}", &input[..12]);
        }
    }
    Ok(())
}
