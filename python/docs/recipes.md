---
title: Recipes
description: Pipelines for common jobs, from Markdown to safe HTML to checking prose.
path: /python/recipes/
icon: chef-hat
order: 5
---

# Recipes

## Markdown to HTML, as GitHub renders it

GFM syntax with its alerts, headings that link to themselves and colored code:

````python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.gfm({"alerts": True}))
    .use(md.to_html())
    .use(html.slug({"autolink": {"behavior": "wrap"}}))
    .use(html.highlight({"theme": "github-dark"}))
    .use(html.stringify())
    .process("## Install\n\n> [!TIP]\n> Use pnpm.\n\n```sh\nnpm install textlane\n```")
)
file["value"]
# <h2 id="install"><a href="#install">Install</a></h2>
# <div class="markdown-alert markdown-alert-tip" dir="auto">
# …
# <pre class="shiki github-dark" style="background-color:#24292e;color:#e1e4e8" …
# …
````

## Untrusted Markdown to safe HTML

Keep the raw HTML a user wrote, then let `html.sanitize` remove whatever GitHub's schema does
not allow:

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.to_html({"allowDangerousHtml": True}))
    .use(html.raw())
    .use(html.sanitize())
    .use(html.stringify())
    .process("Hi <img src=x onerror=alert(1)>")
)
file["value"]
# <p>Hi <img src="x"></p>
```

## HTML to Markdown

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(html.parse({"fragment": True}))
    .use(html.to_markdown())
    .use(md.gfm())
    .use(md.stringify())
    .process("<h1>Notes</h1><ul><li><s>old</s></li><li>new</li></ul>")
)
file["value"]
# # Notes
#
# * ~~old~~
# * new
```

## Formatting Markdown

Parse and stringify with the markers you prefer:

```python
from textlane import md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.stringify({"bullet": "-", "emphasis": "_", "strong": "*"}))
    .process("* *a*\n* __b__")
)
file["value"]
# - _a_
# - **b**
```

## MDX to JavaScript

```python
from textlane import md, mdx, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.mdx())
    .use(md.to_html())
    .use(mdx.compile())
    .process("# Hi {props.name}")
)
file["value"]
# import {jsx as _jsx} from "react/jsx-runtime";
# …
```

## Checking prose in Markdown

The checks report messages and leave the text alone; each message names its rule and where it
is:

```python
from textlane import md, nl, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.to_natural_language())
    .use(nl.passive())
    .use(nl.indefinite_article())
    .use(nl.stringify())
    .process("# Notes\n\nThe build was broken by a update.")
)
report = "\n".join(
    f"{m['place']['start']['line']}:{m['place']['start']['column']} {m['reason']}"
    for m in file["messages"]
)
report
# 3:15 Unexpected use of the passive voice
# 3:25 Unexpected article `a` before `update`, expected `an`
```
