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
