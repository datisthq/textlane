---
title: Recipes
description: Pipelines for common jobs, from Markdown to safe HTML to checking prose.
path: /typescript/recipes/
icon: chef-hat
order: 5
---

# Recipes

## Markdown to HTML, as GitHub renders it

GFM syntax with its alerts, headings that link to themselves and colored code:

````js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.gfm({ alerts: true }))
  .use(md.toHtml())
  .use(html.slug({ autolink: { behavior: "wrap" } }))
  .use(html.highlight({ theme: "github-dark" }))
  .use(html.stringify())
  .processSync("## Install\n\n> [!TIP]\n> Use pnpm.\n\n```sh\nnpm install textlane\n```")
file.value
// <h2 id="install"><a href="#install">Install</a></h2>
// <div class="markdown-alert markdown-alert-tip" dir="auto">
// …
// <pre class="shiki github-dark" style="background-color:#24292e;color:#e1e4e8" …
// …
````

## Untrusted Markdown to safe HTML

Keep the raw HTML a user wrote, then let `html.sanitize` remove whatever GitHub's schema does
not allow:

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.toHtml({ allowDangerousHtml: true }))
  .use(html.raw())
  .use(html.sanitize())
  .use(html.stringify())
  .processSync("Hi <img src=x onerror=alert(1)>")
file.value
// <p>Hi <img src="x"></p>
```

## HTML to Markdown

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = textlane()
  .use(html.parse({ fragment: true }))
  .use(html.toMarkdown())
  .use(md.gfm())
  .use(md.stringify())
  .processSync("<h1>Notes</h1><ul><li><s>old</s></li><li>new</li></ul>")
file.value
// # Notes
//
// * ~~old~~
// * new
```

## Formatting Markdown

Parse and stringify with the markers you prefer:

```js
import { textlane } from "textlane"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.stringify({ bullet: "-", emphasis: "_", strong: "*" }))
  .processSync("* *a*\n* __b__")
file.value
// - _a_
// - **b**
```

## MDX to JavaScript

```js
import { textlane } from "textlane"
import * as md from "textlane/md"
import * as mdx from "textlane/mdx"

const file = textlane()
  .use(md.parse())
  .use(md.mdx())
  .use(md.toHtml())
  .use(mdx.compile())
  .processSync("# Hi {props.name}")
file.value
// import {jsx as _jsx} from "react/jsx-runtime";
// …
```

## Checking prose in Markdown

The checks report messages and leave the text alone; each message names its rule and where it
is:

```js
import { textlane } from "textlane"
import * as md from "textlane/md"
import * as nl from "textlane/nl"

const file = textlane()
  .use(md.parse())
  .use(md.toNaturalLanguage())
  .use(nl.passive())
  .use(nl.indefiniteArticle())
  .use(nl.stringify())
  .processSync("# Notes\n\nThe build was broken by a update.")
const report = file.messages
  .map(message => `${message.place?.start.line}:${message.place?.start.column} ${message.reason}`)
  .join("\n")
report
// 3:15 Unexpected use of the passive voice
// 3:25 Unexpected article `a` before `update`, expected `an`
```
