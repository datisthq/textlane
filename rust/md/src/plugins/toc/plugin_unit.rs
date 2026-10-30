use std::time::{Duration, Instant};

use super::*;
use crate::{Stringify, parse_markdown, stringify_markdown};

fn transform(options: &Toc) -> Result<TocTransform, Error> {
    let heading = options.heading.as_deref().unwrap_or(DEFAULT_HEADING);
    Ok(TocTransform {
        heading: compile_expression(heading)?,
        skip: None,
        parents: options.parents.clone(),
        min_depth: options.min_depth.unwrap_or(0),
        max_depth: options.max_depth.unwrap_or(0),
        tight: options.tight.unwrap_or(true),
        ordered: options.ordered.unwrap_or(false),
        prefix: options.prefix.clone().unwrap_or_default(),
    })
}

fn run_toc(input: &str, options: &Toc) -> Result<String, Error> {
    let alloc = Allocator::new();
    let tree = Tree::Mdast(parse_markdown(input, &alloc));
    let Tree::Mdast(root) = transform(options)?.run(tree, &mut File::new(), &alloc)?
    else {
        return Err(Error::new("invalid-stage", "expected mdast"));
    };
    stringify_markdown(&root, &Stringify::default())
}

#[test]
fn replaces_the_section_under_the_heading() -> Result<(), Error> {
    let output = run_toc(
        "# Guide\n\n## Contents\n\n## Install\n\n### From npm\n\n## Use",
        &Toc::default(),
    )?;
    assert_eq!(
        output,
        "# Guide\n\n## Contents\n\n* [Install](#install)\n  * [From npm](#from-npm)\n* [Use](#use)\n\n## Install\n\n### From npm\n\n## Use\n"
    );
    Ok(())
}

#[test]
fn repeats_children_when_the_closing_heading_comes_first() -> Result<(), Error> {
    let options = Toc {
        parents: Some(ParentTest::Any(vec![
            ParentCheck::Kind("root".into()),
            ParentCheck::Kind("blockquote".into()),
        ])),
        ..Toc::default()
    };
    let output = run_toc("a\n\nb\n\n> c\n>\n> d\n>\n> ## toc\n\n## A\n", &options)?;
    assert!(output.contains("[A](#a)"), "{output}");
    Ok(())
}

#[test]
fn stays_linear_over_many_headings() -> Result<(), Error> {
    let time = |count: usize| -> Result<Duration, Error> {
        let mut input = String::from("## Contents\n\n");
        for index in 0..count {
            input.push_str(if index % 2 == 0 {
                "## Same *x*\n\n"
            } else {
                "### Same\n\n"
            });
        }
        let start = Instant::now();
        let output = run_toc(&input, &Toc::default())?;
        assert!(output.contains("same-x-1"));
        Ok(start.elapsed())
    };
    let small = time(10_000)?;
    let large = time(40_000)?;
    assert!(large < small * 12, "{small:?} then {large:?}");
    Ok(())
}
