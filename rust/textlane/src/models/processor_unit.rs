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
