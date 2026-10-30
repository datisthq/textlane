---
title: Markdown plugins
description: Every Markdown plugin, its options and the unified package it follows.
path: /rust/markdown-plugins/
icon: file-text
order: 6
---

# Markdown plugins

Markdown plugins parse Markdown into mdast, extend its syntax and turn mdast into Markdown, HTML trees or natural-language trees. `md.parse` and `md.stringify` are the parser and the compiler; a syntax extension such as `md.gfm` teaches both, so add it beside either.

The tables list each option as `Pipeline` takes it, by its JSON key and type; each plugin
struct's own fields and Rust types are on [docs.rs](https://docs.rs/textlane).

## md.parse

Parses Markdown into an mdast tree, CommonMark as micromark reads it; add syntax extensions such as `md.gfm` beside it.

```rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::ToHtml::default())
    .with(html::Stringify::default())
    .freeze()?
    .process("# Hello *world*")?;
output.value
// <h1>Hello <em>world</em></h1>
```

<!-- generated:options md.parse -->

Follows [remark-parse](https://www.npmjs.com/package/remark-parse).

No options.

<!-- /generated -->

## md.directive

Adds generic directives — `:text`, `::leaf` and `:::container` — read by `md.parse` and written by `md.stringify`.

```rust
use textlane::{Processor, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Directive::default())
    .with(md::Stringify::default())
    .freeze()?
    .process(":::note{.info}\nRead *this*.\n:::")?;
output.value
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

```rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Gfm::default())
    .with(md::ToHtml::default())
    .with(html::Stringify::default())
    .freeze()?
    .process("~~old~~ www.example.com")?;
output.value
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

```rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::DefinitionList)
    .with(md::ToHtml::default())
    .with(html::Stringify::default())
    .freeze()?
    .process("Apple\n: A fruit.")?;
output.value
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

```rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Math::default())
    .with(md::ToHtml::default())
    .with(html::Stringify::default())
    .freeze()?
    .process("Euler: $e^{i\\pi} = -1$")?;
output.value
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

```rust
use textlane::{Processor, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Mdx::default())
    .with(md::Stringify::default())
    .freeze()?
    .process("import {Chart} from './chart.js'\n\n# Sales {year}\n\n<Chart  data={rows}/>")?;
output.value
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

Reads frontmatter — YAML by default, TOML or your own fences by option — into a node of its own. In Rust its `matters` are the matters themselves: `md::Matter::yaml()`, `md::Matter::toml()` or one of your own.

```rust
use textlane::{Processor, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Frontmatter {
        matters: vec![md::Matter::yaml(), md::Matter::toml()],
        ..Default::default()
    })
    .with(md::Stringify::default())
    .freeze()?
    .process("+++\ntitle = \"Hi\"\n+++\n\n# Hi")?;
output.value
// +++
// title = "Hi"
// +++
//
// # Hi
```

With `data: true` it also parses the YAML a document starts with into the file's `data.matter`, as vfile-matter does (`{}` when there is none; YAML 1.2's core schema, as the `yaml` package reads it), and `strip: true` drops that node from the tree. Invalid YAML fails with `invalid-frontmatter`. To combine them with other matters, pass `{ matters, data, strip }`.

```rust
let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Frontmatter { data: true, ..Default::default() })
    .with(md::Stringify::default())
    .freeze()?
    .process("---\ntitle: Hello\ntags: [a, b]\n---\n\n# Hello")?;
output.file.data["matter"]
// {"title": "Hello", "tags": ["a", "b"]}
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

```rust
use textlane::{Processor, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Gemoji)
    .with(md::Stringify::default())
    .freeze()?
    .process("Shipped :tada:")?;
output.value
// Shipped 🎉
```

<!-- generated:options md.gemoji -->

Follows [remark-gemoji](https://www.npmjs.com/package/remark-gemoji).

No options.

<!-- /generated -->

## md.component

Turns directives, GitHub alerts and other nodes into MDX JSX elements, the components an MDX
runtime renders, by declarative rules, so a site's `:::note` or `::button` needs no plugin of
its own. Each directive rule names the directive and the component, where its name and label
go, which attributes it keeps, what it requires and whether adjacent elements are grouped. Code rules (`codes`) do the same for fenced code: a block becomes a component by its language or a meta `key="value"`, or adjacent blocks are wrapped together, as tabbed code is.
Run it after `md::Directive` and before `md::ToHtml` and `mdx::Compile`, which compile the
elements to component calls.

```rust
use textlane::{Processor, md};

let rule = md::DirectiveRule {
    name: md::Names::One("note".into()),
    component: "Callout".into(),
    name_as: Some("type".into()),
    label_as: Some("title".into()),
    ..Default::default()
};
let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Directive::default())
    .with(md::Mdx::default())
    .with(md::Component { directives: vec![rule], ..Default::default() })
    .with(md::Stringify::default())
    .freeze()?
    .process(":::note[Heads up]\nRead *this*.\n:::")?;
output.value
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

| Option         | Type                      | Default | Description                                                                                                                           |
| -------------- | ------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| `name`         | `Names`                   |         | The directive's name, or several.                                                                                                     |
| `kind`         | `DirectiveKind \| null`   |         | The kind of directive: `text`, `leaf` or `container`; any when left out.                                                              |
| `where`        | `Record<string, string>`  | `{}`    | Attribute values the directive must have.                                                                                             |
| `component`    | `string`                  |         | The component: a text element for a text directive, else a flow element.                                                              |
| `nameAs`       | `string \| null`          | `null`  | The attribute that gets the directive's name.                                                                                         |
| `labelAs`      | `string \| null`          | `null`  | The attribute that gets the label's text: a container's label paragraph or a text or leaf directive's content, which is then dropped. |
| `labelDefault` | `string \| null`          | `null`  | The label's text when there is none.                                                                                                  |
| `attributes`   | `AttributeRule[] \| null` |         | The attributes to keep, in order; every one, as written, when left out.                                                               |
| `required`     | `string[]`                | `[]`    | Attributes without which the directive is left to `unknownDirectives`.                                                                |
| `group`        | `GroupRule \| null`       |         | Wrap runs of adjacent siblings this rule converted in another component.                                                              |

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
equivalent for stays as written. Run it before `md::ToHtml` and `mdx::Compile`.

````rust
use textlane::{Processor, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Mdx::default())
    .with(md::PackageCommand::default())
    .with(md::Stringify::default())
    .freeze()?
    .process("```npm\nnpm install textlane\n```")?;
output.value
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

Reads `{#id}` at the end of a heading, after a space, as its id, as remark-heading-id does; `md.toHtml` puts it on the heading. remark-heading-id's `defaults` is not ported: `html.slug` gives the other headings ids. With `attributes: true` it reads a whole attribute list instead, as `md.directive` reads one: `## API {#api .wide data-step=2}` gives the heading an id, classes and a `data-step`. Only the last braces are read, event handlers (`on…`) and invalid names are dropped, and on untrusted input the result still belongs behind `html.sanitize`.

```rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::HeadingId::default())
    .with(md::ToHtml::default())
    .with(html::Stringify::default())
    .freeze()?
    .process("# Install {#setup}")?;
output.value
// <h1 id="setup">Install</h1>
```

<!-- generated:options md.headingId -->

Follows [remark-heading-id](https://www.npmjs.com/package/remark-heading-id).

| Option       | Type      | Default | Description                                                                                                                                        |
| ------------ | --------- | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `attributes` | `boolean` | `false` | Read a whole attribute list, `{#id .class key=value key="v w"}`, as directives write theirs: classes become `className` and other keys properties. |

<!-- /generated -->

## md.pathRewrite

Rewrites the URLs of images, links and definitions as a site serves them, from where the document lives: a relative URL is resolved against the file's path and made relative to `root`, every root-relative URL gets `base` once, and a link to a Markdown file becomes its page's URL, from `pages` or by `markdownLinks` (`keep`, `directory` or `html`). URLs with a scheme, protocol-relative ones and fragments stay as they are. Pass the document's path with the text.

```rust
use textlane::{File, Processor, html, md};

let rewrite = md::PathRewrite {
    root: Some("/repo/docs".into()),
    base: Some("/site".into()),
    markdown_links: md::MarkdownLinks::Directory,
    ..Default::default()
};
let file = File {
    path: Some("/repo/docs/guide/intro.md".into()),
    ..File::new()
};
let output = Processor::new()
    .with(md::Parse::default())
    .with(rewrite)
    .with(md::ToHtml::default())
    .with(html::Stringify::default())
    .freeze()?
    .process_file("![logo](../logo.png) and [next](./next.md)", file)?;
output.value
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

## md.strip

Strips mdast down to its text, as strip-markdown does: code, HTML, tables and definitions go, headings become paragraphs and formatting gives way to its content, which suits search indexes and plain-text summaries. `keep` leaves node types as they are and `remove` drops more.

```rust
use textlane::{Processor, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Strip::default())
    .with(md::Stringify::default())
    .freeze()?
    .process("# Hello *world*\n\nSee [the docs](/docs).")?;
output.value
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

```rust
use textlane::{Processor, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Toc::default())
    .with(md::Stringify::default())
    .freeze()?
    .process("# Guide\n\n## Contents\n\n## Install\n\n### From npm\n\n## Use")?;
output.value
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

```rust
use textlane::{Processor, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Stringify { bullet: Some("-".into()), emphasis: Some("_".into()), ..Default::default() })
    .freeze()?
    .process("* *one*\n* two")?;
output.value
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

```rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::ToHtml { allow_dangerous_html: true, ..Default::default() })
    .with(html::Stringify { allow_dangerous_html: true, ..Default::default() })
    .freeze()?
    .process("Hi <kbd>Ctrl</kbd>")?;
output.value
// <p>Hi <kbd>Ctrl</kbd></p>
```

<!-- generated:options md.toHtml -->

Follows [remark-rehype](https://www.npmjs.com/package/remark-rehype).

| Option                 | Type             | Default | Description                                                                                     |
| ---------------------- | ---------------- | ------- | ----------------------------------------------------------------------------------------------- |
| `allowDangerousHtml`   | `boolean`        | `false` | Keep raw HTML in the tree instead of dropping it.                                               |
| `clobberPrefix`        | `string \| null` | `null`  | What footnote `id`s start with, so they cannot clobber the page's; `user-content-` when unset.  |
| `footnoteBackContent`  | `string \| null` | `null`  | The text of each footnote's back reference; `↩`, numbered from the second, when unset.          |
| `footnoteBackLabel`    | `string \| null` | `null`  | The `aria-label` of each footnote's back reference; `Back to reference 1` and so on when unset. |
| `footnoteLabel`        | `string \| null` | `null`  | The heading of the footnotes section; `Footnotes` when unset.                                   |
| `footnoteLabelTagName` | `string \| null` | `null`  | The tag name of the footnotes section's heading; `h2` when unset.                               |
| `passThrough`          | `string[]`       | `[]`    | Node types without a handler to copy into hast as they are, their children converted.           |

<!-- /generated -->

## md.toNaturalLanguage

Turns mdast into nlcst, the natural-language tree, so `nl.*` plugins can check the prose of a Markdown document.

```rust
use textlane::{Processor, md, nl};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::ToNaturalLanguage::default())
    .with(nl::RepeatedWords)
    .with(nl::Stringify)
    .freeze()?
    .process("It is *the the* best.")?;
output.file.messages[0].reason
// Unexpected repeated `the`, remove one occurrence
```

<!-- generated:options md.toNaturalLanguage -->

Follows [remark-retext](https://www.npmjs.com/package/remark-retext).

| Option   | Type       | Default | Description                                                                        |
| -------- | ---------- | ------- | ---------------------------------------------------------------------------------- |
| `ignore` | `string[]` | `[]`    | Node types to leave out, besides `table`, `tableRow` and `tableCell`.              |
| `source` | `string[]` | `[]`    | Node types to keep as one `SourceNode` of their source text, besides `inlineCode`. |

<!-- /generated -->
