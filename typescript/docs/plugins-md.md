---
title: Markdown plugins
description: Every Markdown plugin, its options and the unified package it follows.
path: /typescript/markdown-plugins/
icon: file-text
order: 6
---

# Markdown plugins

Markdown plugins parse Markdown into mdast, extend its syntax and turn mdast into Markdown, HTML trees or natural-language trees. `md.parse` and `md.stringify` are the parser and the compiler; a syntax extension such as `md.gfm` teaches both, so add it beside either.

## md.parse

Parses Markdown into an mdast tree, CommonMark as micromark reads it; add syntax extensions such as `md.gfm` beside it.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.toHtml())
  .use(html.stringify())
  .processSync("# Hello *world*")
file.value
// <h1>Hello <em>world</em></h1>
```

<!-- generated:options md.parse -->

Follows [remark-parse](https://www.npmjs.com/package/remark-parse).

No options.

<!-- /generated -->

## md.directive

Adds generic directives — `:text`, `::leaf` and `:::container` — read by `md.parse` and written by `md.stringify`.

```js
import { textlane } from "textlane"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.directive())
  .use(md.stringify())
  .processSync(":::note{.info}\nRead *this*.\n:::")
file.value
// :::note{.info}
// Read *this*.
// :::
```

<!-- generated:options md.directive -->

Follows [remark-directive](https://www.npmjs.com/package/remark-directive).

| Option                    | Type             | Default | Description                                                                     |
| ------------------------- | ---------------- | ------- | ------------------------------------------------------------------------------- |
| `collapseEmptyAttributes` | `boolean`        | `true`  | Write an empty attribute as its name alone, `{a}`, rather than `{a=""}`.        |
| `preferShortcut`          | `boolean`        | `true`  | Write `id` and `class` as `#a` and `.b` where they can be.                      |
| `preferUnquoted`          | `boolean`        | `false` | Leave attribute values unquoted where they can be.                              |
| `quote`                   | `string \| null` | `null`  | The quote of attribute values: `"` or `'`; `md.stringify`'s `quote` when unset. |
| `quoteSmart`              | `boolean`        | `false` | Use the other quote when the preferred one occurs more often in a value.        |

<!-- /generated -->

## md.gfm

Adds GitHub Flavored Markdown: tables, task lists, strikethrough, autolink literals and footnotes.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.gfm())
  .use(md.toHtml())
  .use(html.stringify())
  .processSync("~~old~~ www.example.com")
file.value
// <p><del>old</del> <a href="http://www.example.com">www.example.com</a></p>
```

<!-- generated:options md.gfm -->

Follows [remark-gfm](https://www.npmjs.com/package/remark-gfm).

| Option             | Type      | Default | Description                                                                |
| ------------------ | --------- | ------- | -------------------------------------------------------------------------- |
| `singleTilde`      | `boolean` | `true`  | Let one tilde, `~a~`, form strikethrough, as well as two.                  |
| `tableCellPadding` | `boolean` | `true`  | Pad table cells with a space on each side when writing Markdown.           |
| `tablePipeAlign`   | `boolean` | `true`  | Line up table pipes when writing Markdown.                                 |
| `firstLineBlank`   | `boolean` | `false` | Write a footnote definition's first line blank when it holds flow content. |

<!-- /generated -->

## md.definitionList

Adds definition lists: a term, then lines starting with `: ` that define it.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.definitionList())
  .use(md.toHtml())
  .use(html.stringify())
  .processSync("Apple\n: A fruit.")
file.value
// <dl>
// <dt>Apple</dt>
// <dd>A fruit.
// </dd>
// </dl>
```

<!-- generated:options md.definitionList -->

Follows [remark-definition-list](https://www.npmjs.com/package/remark-definition-list).

No options.

<!-- /generated -->

## md.math

Adds math: `$$` blocks and `$` inline, kept as math nodes for a later stage to render.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.math())
  .use(md.toHtml())
  .use(html.stringify())
  .processSync("Euler: $e^{i\\pi} = -1$")
file.value
// <p>Euler: <code class="language-math math-inline">e^{i\pi} = -1</code></p>
```

<!-- generated:options md.math -->

Follows [remark-math](https://www.npmjs.com/package/remark-math).

| Option                 | Type      | Default | Description                                               |
| ---------------------- | --------- | ------- | --------------------------------------------------------- |
| `singleDollarTextMath` | `boolean` | `true`  | Let one dollar, `$a$`, form math in text, as well as two. |

<!-- /generated -->

## md.mdx

Adds MDX syntax — JSX, `{expressions}` and `import`/`export` — and turns off indented code, autolinks and raw HTML, as MDX does.

```js
import { textlane } from "textlane"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.mdx())
  .use(md.stringify())
  .processSync("import {Chart} from './chart.js'\n\n# Sales {year}\n\n<Chart  data={rows}/>")
file.value
// import {Chart} from './chart.js'
//
// # Sales {year}
//
// <Chart data={rows} />
```

<!-- generated:options md.mdx -->

Follows [remark-mdx](https://www.npmjs.com/package/remark-mdx).

| Option             | Type             | Default | Description                                                                |
| ------------------ | ---------------- | ------- | -------------------------------------------------------------------------- |
| `quote`            | `string \| null` | `null`  | The quote of JSX attribute values: `"` (when unset) or `'`.                |
| `quoteSmart`       | `boolean`        | `false` | Use the other quote when the preferred one occurs more often in a value.   |
| `tightSelfClosing` | `boolean`        | `false` | Write self-closing tags without a space before the slash, `<a/>`.          |
| `printWidth`       | `number \| null` | `null`  | The width past which a flow element's attributes go on lines of their own. |

<!-- /generated -->

## md.frontmatter

Reads frontmatter — YAML by default, TOML or your own fences by option — into a node of its own.

```js
import { textlane } from "textlane"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.frontmatter(["yaml", "toml"]))
  .use(md.stringify())
  .processSync('+++\ntitle = "Hi"\n+++\n\n# Hi')
file.value
// +++
// title = "Hi"
// +++
//
// # Hi
```

With `data: true` it also parses the YAML a document starts with into the file's `data.matter`, as vfile-matter does (`{}` when there is none; YAML 1.2's core schema, as the `yaml` package reads it), and `strip: true` drops that node from the tree. Invalid YAML fails with `invalid-frontmatter`. To combine them with other matters, pass `{ matters, data, strip }`.

```js
const withData = textlane()
  .use(md.parse())
  .use(md.frontmatter({ data: true }))
  .use(md.stringify())
  .processSync("---\ntitle: Hello\ntags: [a, b]\n---\n\n# Hello")
withData.data.matter
// { title: "Hello", tags: ["a", "b"] }
```

<!-- generated:options md.frontmatter -->

Follows [remark-frontmatter](https://www.npmjs.com/package/remark-frontmatter).

The kinds of matter to recognize, as remark-frontmatter takes them (a preset, a matter, or a list of them; `yaml` when unset), or textlane's options object.

Takes `"yaml" | "toml" | Matter | ("yaml" | "toml" | Matter)[] | FrontmatterSettings`.

**`Matter`**

One kind of frontmatter: the node `type` it becomes and its fences, a `marker` repeated three times or a whole `fence`.

| Option     | Type      | Default | Description |
| ---------- | --------- | ------- | ----------- |
| `type`     | `string`  |         |             |
| `marker`   | `Fence`   |         |             |
| `fence`    | `Fence`   |         |             |
| `anywhere` | `boolean` |         |             |

**`FrontmatterSettings`**

textlane's options: the matters, and what to do with YAML frontmatter.

| Option    | Type                                                           | Default | Description                                                                                                               |
| --------- | -------------------------------------------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------- |
| `matters` | `"yaml" \| "toml" \| Matter \| ("yaml" \| "toml" \| Matter)[]` |         | The kinds of matter to recognize, as remark-frontmatter takes them; `yaml` when unset.                                    |
| `data`    | `boolean`                                                      |         | Parse the YAML frontmatter a document starts with into `file.data.matter`, `{}` when there is none, as vfile-matter does. |
| `strip`   | `boolean`                                                      |         | Remove the YAML frontmatter a document starts with from the tree.                                                         |

**`Fence`**

A fence, or its opening and closing ones.

Takes `string | FencePair`.

**`FencePair`**

Different opening and closing fences.

| Option  | Type     | Default | Description |
| ------- | -------- | ------- | ----------- |
| `open`  | `string` |         |             |
| `close` | `string` |         |             |

<!-- /generated -->

## md.gemoji

Turns gemoji shortcodes such as `:tada:` into the emoji they name.

```js
import { textlane } from "textlane"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.gemoji())
  .use(md.stringify())
  .processSync("Shipped :tada:")
file.value
// Shipped 🎉
```

<!-- generated:options md.gemoji -->

Follows [remark-gemoji](https://www.npmjs.com/package/remark-gemoji).

No options.

<!-- /generated -->

## md.component

Turns directives, GitHub alerts and other nodes into MDX JSX elements, the components an MDX
runtime renders, by declarative rules, so a site's `:::note` or `::button` needs no
JavaScript plugin. Each directive rule names the directive and the component, where its name
and label go, which attributes it keeps, what it requires and whether adjacent elements are
grouped, as livemark's cards and tabs are. Code rules (`codes`) do the same for fenced code: a block becomes a component by its language or a meta `key="value"`, or adjacent blocks are wrapped together, as tabbed code is. A container rule's `treeAs` reads its lists as a file tree, as Starlight's `<FileTree>` reads one, into a JSON attribute of items `{ name, type, comment?, highlighted?, children? }`: a name ending in `/` or holding a list is a directory, a **bold** name is highlighted, text after the name is its comment and `…` stands for more files. Run it after `md.directive` and before `md.toHtml`
and `mdx.compile`, which compile the elements to component calls.

```js
import { textlane } from "textlane"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.directive())
  .use(md.mdx())
  .use(
    md.component({
      directives: [{ name: "note", component: "Callout", nameAs: "type", labelAs: "title" }],
    }),
  )
  .use(md.stringify())
  .processSync(":::note[Heads up]\nRead *this*.\n:::")
file.value
// <Callout type="note" title="Heads up">
//   Read *this*.
// </Callout>
```

<!-- generated:options md.component -->

| Option              | Type                     | Default | Description                                                            |
| ------------------- | ------------------------ | ------- | ---------------------------------------------------------------------- |
| `directives`        | `DirectiveRule[]`        |         | Rules for directives; the first that matches a directive converts it.  |
| `alerts`            | `AlertRule \| null`      |         | GitHub alerts, `> [!NOTE]`, into a component.                          |
| `nodes`             | `Record<string, string>` | `{}`    | Node types, such as `defList`, into components keeping their children. |
| `unknownDirectives` | `UnknownDirectives`      |         | What becomes of a directive no rule converts.                          |
| `codes`             | `CodeRule[]`             |         | Rules for fenced code; the first that matches a block applies.         |

**`DirectiveRule`**

How `md.component` turns matching directives into one component.

| Option         | Type                      | Default | Description                                                                                                                                                           |
| -------------- | ------------------------- | ------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `name`         | `Names`                   |         | The directive's name, or several.                                                                                                                                     |
| `kind`         | `DirectiveKind \| null`   |         | The kind of directive: `text`, `leaf` or `container`; any when left out.                                                                                              |
| `where`        | `Record<string, string>`  | `{}`    | Attribute values the directive must have.                                                                                                                             |
| `component`    | `string`                  |         | The component: a text element for a text directive, else a flow element.                                                                                              |
| `nameAs`       | `string \| null`          | `null`  | The attribute that gets the directive's name.                                                                                                                         |
| `labelAs`      | `string \| null`          | `null`  | The attribute that gets the label's text: a container's label paragraph or a text or leaf directive's content, which is then dropped.                                 |
| `labelDefault` | `string \| null`          | `null`  | The label's text when there is none.                                                                                                                                  |
| `attributes`   | `AttributeRule[] \| null` |         | The attributes to keep, in order; every one, as written, when left out.                                                                                               |
| `required`     | `string[]`                | `[]`    | Attributes without which the directive is left to `unknownDirectives`.                                                                                                |
| `group`        | `GroupRule \| null`       |         | Wrap runs of adjacent siblings this rule converted in another component.                                                                                              |
| `treeAs`       | `string \| null`          | `null`  | The attribute that gets the lists of a container directive as a file tree, JSON items `{name, type, comment?, highlighted?, children?}`; the lists leave its content. |

**`AlertRule`**

How `md.component` turns GitHub alerts, a block quote opening with `[!TYPE]`, into a component.

| Option      | Type                     | Default | Description                                                                           |
| ----------- | ------------------------ | ------- | ------------------------------------------------------------------------------------- |
| `component` | `string`                 |         | The component an alert becomes.                                                       |
| `typeAs`    | `string`                 |         | The attribute that gets the alert's type.                                             |
| `types`     | `Record<string, string>` |         | Each alert type, as written, to the value of `typeAs`; other types stay block quotes. |

**`UnknownDirectives`**

What `md.component` does with a directive no rule converts.

Takes `"keep" | "source"`.

**`CodeRule`**

How `md.component` turns matching code blocks into a component, or groups them.

| Option       | Type                    | Default | Description                                                                      |
| ------------ | ----------------------- | ------- | -------------------------------------------------------------------------------- |
| `lang`       | `Names \| null`         |         | The block's language, or several; any when left out.                             |
| `meta`       | `string \| null`        | `null`  | A `key="value"` the block's meta must hold.                                      |
| `component`  | `string \| null`        | `null`  | The component the block becomes; left out, the block stays code and only groups. |
| `valueAs`    | `string \| null`        | `null`  | The attribute that gets the code; left out, the code is the component's text.    |
| `attributes` | `AttributeRule[]`       |         | Meta `key="value"` pairs to keep as attributes, in order.                        |
| `group`      | `CodeGroupRule \| null` |         | Wrap runs of adjacent blocks this rule matched in another component.             |

**`Names`**

One name or several.

Takes `string | string[]`.

**`DirectiveKind`**

A directive's kind, by how many colons open it.

Takes `"text" | "leaf" | "container"`.

**`AttributeRule`**

An attribute to keep: its name, or its name with what to call it and a default.

Takes `string | { name: string; rename?: string | null; default?: string | null }`.

**`GroupRule`**

The component wrapping a run of adjacent siblings one rule converted.

| Option      | Type             | Default | Description                                                 |
| ----------- | ---------------- | ------- | ----------------------------------------------------------- |
| `component` | `string`         |         | The wrapping component.                                     |
| `min`       | `number \| null` | `null`  | The shortest run wrapped; 2 when left out.                  |
| `inherit`   | `string[]`       | `[]`    | Attributes the wrapper copies from the run's first element. |

**`CodeGroupRule`**

The component wrapping a run of adjacent code blocks one rule matched; the meta keys it reads are taken out of each block's meta.

| Option      | Type             | Default | Description                                                                        |
| ----------- | ---------------- | ------- | ---------------------------------------------------------------------------------- |
| `component` | `string`         |         | The wrapping component.                                                            |
| `min`       | `number \| null` | `null`  | The shortest run wrapped; 2 when left out.                                         |
| `collect`   | `string \| null` | `null`  | The attribute that gets the run's values of the rule's `meta` key, as a JSON list. |
| `inherit`   | `string[]`       | `[]`    | Meta keys the wrapper copies from the run's first block as attributes.             |

<!-- /generated -->

## md.packageCommand

Turns a fenced block of package-manager commands into one MDX JSX element with the commands
converted for each manager, as `@docusaurus/remark-plugin-npm2yarn` does, so a site's install
tabs need no JavaScript plugin. The block's language names the ecosystem: `npm` converts as
the npm-to-yarn package does, for pnpm, yarn, bun, deno and nub; `pip` for uv, poetry and pdm;
`cargo` for cargo-binstall. Each manager becomes a string attribute named after it, in the
order `ecosystems` lists them, and each line converts on its own; a line a manager has no
equivalent for stays as written. Run it before `md.toHtml` and `mdx.compile`.

````js
import { textlane } from "textlane"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.mdx())
  .use(md.packageCommand())
  .use(md.stringify())
  .processSync("```npm\nnpm install textlane\n```")
file.value
// <PackageTabs npm="npm install textlane" pnpm="pnpm add textlane" yarn="yarn add textlane" bun="bun add textlane" />
````

<!-- generated:options md.packageCommand -->

Follows [npm-to-yarn](https://www.npmjs.com/package/npm-to-yarn).

| Option       | Type         | Default         | Description                                                                          |
| ------------ | ------------ | --------------- | ------------------------------------------------------------------------------------ |
| `component`  | `string`     | `"PackageTabs"` | The component a block becomes; one string attribute per manager.                     |
| `ecosystems` | `Ecosystems` |                 | Each ecosystem's managers, in attribute order; an empty list leaves its blocks code. |

**`Ecosystems`**

The managers `md.packageCommand` converts each ecosystem's commands for.

| Option  | Type             | Default | Description                                                                                       |
| ------- | ---------------- | ------- | ------------------------------------------------------------------------------------------------- |
| `npm`   | `NpmManager[]`   |         | Managers for `npm` blocks, converted as npm-to-yarn converts; npm, pnpm, yarn and bun by default. |
| `pip`   | `PipManager[]`   |         | Managers for `pip` blocks; pip, uv, poetry and pdm by default.                                    |
| `cargo` | `CargoManager[]` |         | Managers for `cargo` blocks; cargo and cargo-binstall by default.                                 |

**`NpmManager`**

A JavaScript package manager an `npm` command converts to.

Takes `"npm" | "pnpm" | "yarn" | "bun" | "deno" | "nub"`.

**`PipManager`**

A Python package manager a `pip` command converts to.

Takes `"pip" | "uv" | "poetry" | "pdm"`.

**`CargoManager`**

A Rust installer a `cargo` command converts to.

Takes `"cargo" | "cargo-binstall"`.

<!-- /generated -->

## md.headingId

Reads `{#id}` at the end of a heading, after a space, as its id, as remark-heading-id does; `md.toHtml` puts it on the heading. remark-heading-id's `defaults` is not ported: `html.slug` gives the other headings ids. With `attributes: true` it reads a whole attribute list instead, as `md.directive` reads one: `## API {#api .wide data-step=2}` gives the heading an id, classes and a `data-step`. Only the last braces are read, event handlers (`on…`) and invalid names are dropped, and on untrusted input the result still belongs behind `html.sanitize`. `markers` turns literal heading suffixes into attributes and strips them, before or after an attribute list: `{"[step]": {"data-step": true}, "[!toc]": {"data-toc-hidden": true}}` reads `## Install [step]` as `<h2 data-step>Install</h2>`.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.headingId())
  .use(md.toHtml())
  .use(html.stringify())
  .processSync("# Install {#setup}")
file.value
// <h1 id="setup">Install</h1>
```

<!-- generated:options md.headingId -->

Follows [remark-heading-id](https://www.npmjs.com/package/remark-heading-id).

| Option       | Type                                          | Default | Description                                                                                                                                                           |
| ------------ | --------------------------------------------- | ------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `attributes` | `boolean`                                     | `false` | Read a whole attribute list, `{#id .class key=value key="v w"}`, as directives write theirs: classes become `className` and other keys properties.                    |
| `markers`    | `Record<string, Record<string, MarkerValue>>` |         | Literal suffixes, such as `[step]`, stripped from a heading's end for the attributes each names, a value or `true` for a bare one: `{"[step]": {"data-step": true}}`. |

**`MarkerValue`**

An attribute a heading marker sets: its value, or `true` for the bare attribute.

Takes `boolean | string`.

<!-- /generated -->

## md.pathRewrite

Rewrites the URLs of images, links and definitions as a site serves them, from where the document lives: a relative URL is resolved against the file's path and made relative to `root`, every root-relative URL gets `base` once, and a link to a Markdown file becomes its page's URL, from `pages` or by `markdownLinks` (`keep`, `directory` or `html`). URLs with a scheme, protocol-relative ones and fragments stay as they are. Pass the document's path with the text.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.pathRewrite({ root: "/repo/docs", base: "/site", markdownLinks: "directory" }))
  .use(md.toHtml())
  .use(html.stringify())
  .processSync({
    value: "![logo](../logo.png) and [next](./next.md)",
    path: "/repo/docs/guide/intro.md",
  })
file.value
// <p><img src="/site/logo.png" alt="logo"> and <a href="/site/guide/next/">next</a></p>
```

<!-- generated:options md.pathRewrite -->

| Option          | Type                     | Default | Description                                                                                                                   |
| --------------- | ------------------------ | ------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `root`          | `string \| null`         | `null`  | The directory URLs count from, as the file's path spells it; the path's own starting point when left out.                     |
| `base`          | `string \| null`         | `null`  | The URL the site is served at, such as `/docs`, put before every root-relative URL once.                                      |
| `nodes`         | `UrlNode[] \| null`      |         | The nodes whose URLs are rewritten; images, links and definitions when left out.                                              |
| `pages`         | `Record<string, string>` | `{}`    | Markdown files, relative to `root`, to the URL of their page; a link to one of them takes it, before `markdownLinks` applies. |
| `markdownLinks` | `MarkdownLinks`          |         | How a link to a Markdown file no page names becomes a URL.                                                                    |

**`UrlNode`**

A node with a URL.

Takes `"image" | "link" | "definition"`.

**`MarkdownLinks`**

How a link to a Markdown file becomes a URL.

Takes `"keep" | "directory" | "html"`.

<!-- /generated -->

## md.shiftHeading

Moves every heading's depth by `shift`, kept within 1 to 6, as rehype-shift-heading does for hast: `-1` makes each h2 an h1. `dropTitle` first drops the document's first node when it is an h1, looking past frontmatter and ESM, which suits a page whose title is rendered on its own; dropping before shifting keeps a document that opens on an h2.

```js
import { textlane } from "textlane"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.shiftHeading({ shift: -1, dropTitle: true }))
  .use(md.stringify())
  .processSync("# 0.2.0\n\n### Features\n\n- a")
file.value
// ## Features
//
// * a
```

<!-- generated:options md.shiftHeading -->

Follows [rehype-shift-heading](https://www.npmjs.com/package/rehype-shift-heading).

| Option      | Type      | Default | Description                                                                        |
| ----------- | --------- | ------- | ---------------------------------------------------------------------------------- |
| `shift`     | `number`  | `0`     | The levels added to each heading's depth: `-1` makes an h2 an h1, `1` an h1 an h2. |
| `dropTitle` | `boolean` | `false` | Drop the document's first node when it is an h1, looking past frontmatter and ESM. |

<!-- /generated -->

## md.strip

Strips mdast down to its text, as strip-markdown does: code, HTML, tables and definitions go, headings become paragraphs and formatting gives way to its content, which suits search indexes and plain-text summaries. `keep` leaves node types as they are and `remove` drops more.

```js
import { textlane } from "textlane"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.strip())
  .use(md.stringify())
  .processSync("# Hello *world*\n\nSee [the docs](/docs).")
file.value
// Hello world
//
// See the docs.
```

<!-- generated:options md.strip -->

Follows [strip-markdown](https://www.npmjs.com/package/strip-markdown).

| Option   | Type       | Default | Description                                             |
| -------- | ---------- | ------- | ------------------------------------------------------- |
| `keep`   | `string[]` | `[]`    | Node types to leave as they are, such as `inlineCode`.  |
| `remove` | `string[]` | `[]`    | Node types to drop, besides those strip-markdown drops. |

<!-- /generated -->

## md.toc

Generates a table of contents, as remark-toc does: under the first heading matching `heading` (`Contents`, `Table of contents` or `TOC` by default), it replaces whatever sat before the next heading as deep or shallower with a list linking to every heading after, by the ids github-slugger makes from their text, or the ids `md.headingId` stored. `heading` and `skip` are JavaScript regular expression sources matched against a heading's whole text, case-insensitively, run by Rust's `regex`: lookaround and back references are refused, and case folding is Unicode's rather than JavaScript's. `parents` takes unist-util-is tests written as JSON (a type, an object of fields, a list), not functions.

```js
import { textlane } from "textlane"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.toc())
  .use(md.stringify())
  .processSync("# Guide\n\n## Contents\n\n## Install\n\n### From npm\n\n## Use")
file.value
// # Guide
//
// ## Contents
//
// * [Install](#install)
//   * [From npm](#from-npm)
// * [Use](#use)
//
// ## Install
//
// ### From npm
//
// ## Use
```

<!-- generated:options md.toc -->

Follows [remark-toc](https://www.npmjs.com/package/remark-toc).

| Option     | Type                 | Default | Description                                                                                                                                                                    |
| ---------- | -------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `heading`  | `string \| null`     | `null`  | The heading to put the table under, as JavaScript regular expression source matched against its whole text, case-insensitively; `(table[ -]of[ -])?contents?\|toc` when unset. |
| `maxDepth` | `number \| null`     | `null`  | The deepest heading rank to list, 1 to 6.                                                                                                                                      |
| `minDepth` | `number \| null`     | `null`  | The shallowest heading rank to list, 1 to 6.                                                                                                                                   |
| `skip`     | `string \| null`     | `null`  | Headings to leave out, as JavaScript regular expression source matched against their whole text, case-insensitively.                                                           |
| `tight`    | `boolean \| null`    | `null`  | Keep the list compact, without blank lines between items; `true` when unset.                                                                                                   |
| `ordered`  | `boolean \| null`    | `null`  | Number the list instead of bulleting it.                                                                                                                                       |
| `prefix`   | `string \| null`     | `null`  | What each link's fragment starts with, such as `user-content-`.                                                                                                                |
| `parents`  | `ParentTest \| null` |         | The parents whose headings are listed, as a unist-util-is test: a node type, an object of fields a parent must equal, or a list of these; only the root when unset.            |

**`ParentTest`**

A unist-util-is test of a node, written as JSON: one check, or a list any of which passes.

Takes `ParentCheck | ParentCheck[]`.

**`ParentCheck`**

One unist-util-is check: a node type, or fields a node must equal.

Takes `string | Record<string, unknown>`.

<!-- /generated -->

## md.stringify

Serializes mdast back into Markdown; its options pick the markers and spacing it writes.

```js
import { textlane } from "textlane"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.stringify({ bullet: "-", emphasis: "_" }))
  .processSync("* *one*\n* two")
file.value
// - _one_
// - two
```

<!-- generated:options md.stringify -->

Follows [remark-stringify](https://www.npmjs.com/package/remark-stringify).

| Option                | Type              | Default | Description                                                 |
| --------------------- | ----------------- | ------- | ----------------------------------------------------------- |
| `bullet`              | `string \| null`  | `null`  | The marker of unordered list items: `*`, `+` or `-`.        |
| `bulletOrdered`       | `string \| null`  | `null`  | The marker of ordered list items: `.` or `)`.               |
| `bulletOther`         | `string \| null`  | `null`  | The marker of a list right after a list with `bullet`.      |
| `closeAtx`            | `boolean`         | `false` | Close ATX headings with as many `#` as they open with.      |
| `emphasis`            | `string \| null`  | `null`  | The marker of emphasis: `*` or `_`.                         |
| `fence`               | `string \| null`  | `null`  | The marker of fenced code: `` ` `` or `~`.                  |
| `fences`              | `boolean`         | `true`  | Fence code; `false` indents it where it can.                |
| `incrementListMarker` | `boolean`         | `true`  | Number ordered list items up from `start`.                  |
| `listItemIndent`      | `string \| null`  | `null`  | How list item content is indented: `one`, `tab` or `mixed`. |
| `quote`               | `string \| null`  | `null`  | The marker of titles: `"` or `'`.                           |
| `resourceLink`        | `boolean`         | `false` | Write links as resources even where an autolink would do.   |
| `rule`                | `string \| null`  | `null`  | The marker of thematic breaks: `*`, `-` or `_`.             |
| `ruleRepetition`      | `number \| null`  | `null`  | How many markers a thematic break has, 3 or more.           |
| `ruleSpaces`          | `boolean`         | `false` | Put spaces between the markers of a thematic break.         |
| `setext`              | `boolean`         | `false` | Write headings of rank 1 and 2 as setext headings.          |
| `strong`              | `string \| null`  | `null`  | The marker of strong: `*` or `_`.                           |
| `tightDefinitions`    | `boolean`         | `false` | Join definitions with no blank line between them.           |
| `unsafe`              | `UnsafePattern[]` |         | Extra patterns of characters to escape.                     |

**`UnsafePattern`**

A character to escape where it would otherwise start or end syntax.

| Option           | Type             | Default | Description                                                      |
| ---------------- | ---------------- | ------- | ---------------------------------------------------------------- |
| `character`      | `string`         | `""`    | The character, escaped as a regular expression would need it.    |
| `before`         | `string \| null` | `null`  | A regular expression that must match right before the character. |
| `after`          | `string \| null` | `null`  | A regular expression that must match right after the character.  |
| `atBreak`        | `boolean`        | `false` | Only at the start of a line, after any indent.                   |
| `inConstruct`    | `Names \| null`  |         | Only inside one of these constructs.                             |
| `notInConstruct` | `Names \| null`  |         | Never inside one of these constructs.                            |
| `beforeNode`     | `Names \| null`  |         | Only at the start of a node right after one of these node types. |
| `afterNode`      | `Names \| null`  |         | Only at the end of a node right before one of these node types.  |
| `unicode`        | `boolean`        | `false` | Read `before` and `after` with Unicode classes such as `\p{L}`.  |

**`Names`**

One name or several.

Takes `string | string[]`.

<!-- /generated -->

## md.toHtml

Turns mdast into hast, the HTML tree, so HTML plugins and `html.stringify` can follow.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.toHtml({ allowDangerousHtml: true }))
  .use(html.stringify({ allowDangerousHtml: true }))
  .processSync("Hi <kbd>Ctrl</kbd>")
file.value
// <p>Hi <kbd>Ctrl</kbd></p>
```

`taskListLabels: { checked, unchecked }` gives each GFM task-list checkbox an `aria-label`, so screen readers name it; remark-rehype writes none, so it is off by default.

<!-- generated:options md.toHtml -->

Follows [remark-rehype](https://www.npmjs.com/package/remark-rehype).

| Option                 | Type                     | Default | Description                                                                                                               |
| ---------------------- | ------------------------ | ------- | ------------------------------------------------------------------------------------------------------------------------- |
| `allowDangerousHtml`   | `boolean`                | `false` | Keep raw HTML in the tree instead of dropping it.                                                                         |
| `clobberPrefix`        | `string \| null`         | `null`  | What footnote `id`s start with, so they cannot clobber the page's; `user-content-` when unset.                            |
| `footnoteBackContent`  | `string \| null`         | `null`  | The text of each footnote's back reference; `↩`, numbered from the second, when unset.                                    |
| `footnoteBackLabel`    | `string \| null`         | `null`  | The `aria-label` of each footnote's back reference; `Back to reference 1` and so on when unset.                           |
| `footnoteLabel`        | `string \| null`         | `null`  | The heading of the footnotes section; `Footnotes` when unset.                                                             |
| `footnoteLabelTagName` | `string \| null`         | `null`  | The tag name of the footnotes section's heading; `h2` when unset.                                                         |
| `passThrough`          | `string[]`               | `[]`    | Node types without a handler to copy into hast as they are, their children converted.                                     |
| `taskListLabels`       | `TaskListLabels \| null` |         | An `aria-label` for each task-list checkbox, so screen readers name it; none when left out, as remark-rehype writes none. |

**`TaskListLabels`**

The labels of task-list checkboxes.

| Option      | Type     | Default | Description                                           |
| ----------- | -------- | ------- | ----------------------------------------------------- |
| `checked`   | `string` |         | The label of a checked box, such as `Completed task`. |
| `unchecked` | `string` |         | The label of an unchecked box, such as `Task`.        |

<!-- /generated -->

## md.toNaturalLanguage

Turns mdast into nlcst, the natural-language tree, so `nl.*` plugins can check the prose of a Markdown document.

```js
import { textlane } from "textlane"
import * as md from "textlane/md"
import * as nl from "textlane/nl"

const file = textlane()
  .use(md.parse())
  .use(md.toNaturalLanguage())
  .use(nl.repeatedWords())
  .use(nl.stringify())
  .processSync("It is *the the* best.")
file.messages[0].reason
// Unexpected repeated `the`, remove one occurrence
```

<!-- generated:options md.toNaturalLanguage -->

Follows [remark-retext](https://www.npmjs.com/package/remark-retext).

| Option   | Type       | Default | Description                                                                        |
| -------- | ---------- | ------- | ---------------------------------------------------------------------------------- |
| `ignore` | `string[]` | `[]`    | Node types to leave out, besides `table`, `tableRow` and `tableCell`.              |
| `source` | `string[]` | `[]`    | Node types to keep as one `SourceNode` of their source text, besides `inlineCode`. |

<!-- /generated -->
