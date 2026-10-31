---
title: Recipes
description: Pipelines for common jobs, from Markdown to safe HTML to checking prose.
path: /rust/recipes/
icon: chef-hat
order: 5
---

# Recipes

## Markdown to HTML, as GitHub renders it

GFM syntax with its alerts, headings that link to themselves and colored code:

````rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Gfm { alerts: Some(md::Alerts::Enabled(true)), ..Default::default() })
    .with(md::ToHtml::default())
    .with(html::Slug {
        autolink: Some(html::Autolink::Options(Box::new(html::AutolinkOptions {
            behavior: html::AutolinkBehavior::Wrap,
            ..Default::default()
        }))),
        ..Default::default()
    })
    .with(html::Highlight { theme: Some("github-dark".into()), ..Default::default() })
    .with(html::Stringify::default())
    .freeze()?
    .process("## Install\n\n> [!TIP]\n> Use pnpm.\n\n```sh\nnpm install textlane\n```")?;
output.value
// <h2 id="install"><a href="#install">Install</a></h2>
// <div class="markdown-alert markdown-alert-tip" dir="auto">
// …
// <pre class="shiki github-dark" style="background-color:#24292e;color:#e1e4e8" …
// …
````

## Untrusted Markdown to safe HTML

Keep the raw HTML a user wrote, then let `html::Sanitize` remove whatever GitHub's schema
does not allow:

```rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::ToHtml { allow_dangerous_html: true, ..Default::default() })
    .with(html::Raw::default())
    .with(html::Sanitize::default())
    .with(html::Stringify::default())
    .freeze()?
    .process("Hi <img src=x onerror=alert(1)>")?;
output.value
// <p>Hi <img src="x"></p>
```

## HTML to Markdown

```rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(html::Parse { fragment: true, ..Default::default() })
    .with(html::ToMarkdown::default())
    .with(md::Gfm::default())
    .with(md::Stringify::default())
    .freeze()?
    .process("<h1>Notes</h1><ul><li><s>old</s></li><li>new</li></ul>")?;
output.value
// # Notes
//
// * ~~old~~
// * new
```

## Formatting Markdown

Parse and stringify with the markers you prefer:

```rust
use textlane::{Processor, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Stringify {
        bullet: Some("-".into()),
        emphasis: Some("_".into()),
        strong: Some("*".into()),
        ..Default::default()
    })
    .freeze()?
    .process("* *a*\n* __b__")?;
output.value
// - _a_
// - **b**
```

## MDX to JavaScript

```rust
use textlane::{Processor, md, mdx};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Mdx::default())
    .with(md::ToHtml::default())
    .with(mdx::Compile::default())
    .freeze()?
    .process("# Hi {props.name}")?;
output.value
// import {jsx as _jsx} from "react/jsx-runtime";
// …
```

## Checking prose in Markdown

The checks report messages and leave the text alone; each message names its rule and where it
is:

```rust
use textlane::{Processor, md, nl};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::ToNaturalLanguage::default())
    .with(nl::Passive::default())
    .with(nl::IndefiniteArticle)
    .with(nl::Stringify)
    .freeze()?
    .process("# Notes\n\nThe build was broken by a update.")?;
let mut report = Vec::new();
for message in &output.file.messages {
    let start = message.place.as_ref().map(|place| (place.start.line, place.start.column));
    let (line, column) = start.unwrap_or_default();
    report.push(format!("{line}:{column} {}", message.reason));
}
report.join("\n")
// 3:15 Unexpected use of the passive voice
// 3:25 Unexpected article `a` before `update`, expected `an`
```
