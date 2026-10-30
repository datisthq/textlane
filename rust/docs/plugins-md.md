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

Reads frontmatter — YAML by default, TOML or your own fences by option — into a node of its own. In Rust it holds the matters themselves: `md::Matter::yaml()`, `md::Matter::toml()` or one of your own.

```rust
use textlane::{Processor, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Frontmatter(vec![md::Matter::yaml(), md::Matter::toml()]))
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

<!-- generated:options md.frontmatter -->

Follows [remark-frontmatter](https://www.npmjs.com/package/remark-frontmatter).

The kinds of matter to recognize, as remark-frontmatter takes them: a preset, a matter, or a list of them; `yaml` when unset.

Takes `"yaml" | "toml" | Matter | ("yaml" | "toml" | Matter)[]`.

**`Matter`**

One kind of frontmatter: the node `type` it becomes and its fences, a `marker` repeated three times or a whole `fence`.

| Option     | Type      | Default | Description |
| ---------- | --------- | ------- | ----------- |
| `type`     | `string`  |         |             |
| `marker`   | `Fence`   |         |             |
| `fence`    | `Fence`   |         |             |
| `anywhere` | `boolean` |         |             |

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
go, which attributes it keeps, what it requires and whether adjacent elements are grouped.
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
