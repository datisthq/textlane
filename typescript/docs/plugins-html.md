---
title: HTML plugins
description: Every HTML plugin, its options and the unified package it follows.
path: /typescript/html-plugins/
icon: code
order: 7
---

# HTML plugins

HTML plugins parse HTML into hast, transform it and turn it into HTML, Markdown trees or natural-language trees. A Markdown pipeline reaches them through `md.toHtml`.

## html.parse

Parses HTML into a hast tree, as a browser does: a document by default, a fragment by option.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"

const file = textlane()
  .use(html.parse({ fragment: true }))
  .use(html.stringify())
  .processSync("<p>One<p>Two")
file.value
// <p>One</p><p>Two</p>
```

<!-- generated:options html.parse -->

Follows [rehype-parse](https://www.npmjs.com/package/rehype-parse).

| Option            | Type      | Default | Description                                                               |
| ----------------- | --------- | ------- | ------------------------------------------------------------------------- |
| `fragment`        | `boolean` | `false` | Parse a fragment, as in a `template`, rather than a whole document.       |
| `emitParseErrors` | `boolean` | `false` | Report parse errors as messages on the file.                              |
| `verbose`         | `boolean` | `false` | Add the positions of opening and closing tags and attributes to elements. |

<!-- /generated -->

## html.raw

Reparses the raw HTML a Markdown document carries, so it becomes real hast nodes.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.toHtml({ allowDangerousHtml: true }))
  .use(html.raw())
  .use(html.stringify())
  .processSync("<details><summary>More</summary>\n\n*Hidden*\n\n</details>")
file.value
// <details><summary>More</summary>
// <p><em>Hidden</em></p>
// </details>
```

<!-- generated:options html.raw -->

Follows [rehype-raw](https://www.npmjs.com/package/rehype-raw).

| Option        | Type       | Default | Description                                                                                         |
| ------------- | ---------- | ------- | --------------------------------------------------------------------------------------------------- |
| `passThrough` | `string[]` | `[]`    | Node types, unknown to hast, to copy through as they are; their children are reparsed on their own. |
| `tagfilter`   | `boolean`  | `false` | Escape the opening `<` of the tags GFM's tagfilter disallows in raw nodes.                          |

<!-- /generated -->

## html.format

Indents the HTML and collapses whitespace, for output a person reads.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"

const file = textlane()
  .use(html.parse({ fragment: true }))
  .use(html.format())
  .use(html.stringify())
  .processSync("<ul><li>One</li><li>Two</li></ul>")
file.value
// <ul>
//   <li>One</li>
//   <li>Two</li>
// </ul>
```

<!-- generated:options html.format -->

Follows [rehype-format](https://www.npmjs.com/package/rehype-format).

| Option          | Type              | Default | Description                                                                                |
| --------------- | ----------------- | ------- | ------------------------------------------------------------------------------------------ |
| `blanks`        | `string[]`        | `[]`    | Tag names of elements to put a blank line between, when both are one.                      |
| `indent`        | `Indent \| null`  |         | One level of indentation: a number of spaces or the text itself; two spaces when left out. |
| `indentInitial` | `boolean \| null` | `null`  | Indent the first level, usually `<html>`'s `head` and `body`; `true` when left out.        |

**`Indent`**

One level of indentation.

Takes `number | string`.

<!-- /generated -->

## html.sanitize

Keeps only the elements and attributes a schema allows — GitHub's, by default — so untrusted HTML is safe to show.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"

const file = textlane()
  .use(html.parse({ fragment: true }))
  .use(html.sanitize())
  .use(html.stringify())
  .processSync('<a href="javascript:alert(1)" onclick="x()">Hi</a><script>x()</script>')
file.value
// <a>Hi</a>
```

<!-- generated:options html.sanitize -->

Follows [rehype-sanitize](https://www.npmjs.com/package/rehype-sanitize).

| Option          | Type                                                | Default | Description                                                                                                 |
| --------------- | --------------------------------------------------- | ------- | ----------------------------------------------------------------------------------------------------------- |
| `allowComments` | `boolean \| null`                                   | `null`  | Keep comments.                                                                                              |
| `allowDoctypes` | `boolean \| null`                                   | `null`  | Keep doctypes.                                                                                              |
| `ancestors`     | `Record<string, string[]> \| null`                  | `null`  | Elements allowed only inside one of these ancestors, by tag name.                                           |
| `attributes`    | `Record<string, AttributeDefinition[]> \| null`     |         | Properties allowed per tag name, `*` for every element; a list names a property and the values it may have. |
| `clobber`       | `string[] \| null`                                  | `null`  | Properties whose values get `clobber_prefix`, so they cannot clobber globals.                               |
| `clobberPrefix` | `string \| null`                                    | `null`  | The prefix of clobbered values.                                                                             |
| `protocols`     | `Record<string, string[]> \| null`                  | `null`  | URL properties and the protocols they may use.                                                              |
| `required`      | `Record<string, Record<string, Primitive>> \| null` |         | Properties set on elements whatever they had, by tag name.                                                  |
| `strip`         | `string[] \| null`                                  | `null`  | Elements dropped with their content rather than unwrapped.                                                  |
| `tagNames`      | `string[] \| null`                                  | `null`  | The elements kept.                                                                                          |

**`AttributeDefinition`**

A property a sanitize schema allows: by name, or a name and its allowed values.

Takes `string | Primitive[]`.

**`Primitive`**

A primitive property value.

Takes `boolean | number | string`.

<!-- /generated -->

## html.slug

Gives each heading an `id` from its text, as GitHub does, so it can be linked to.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.toHtml())
  .use(html.slug())
  .use(html.stringify())
  .processSync("## Getting started")
file.value
// <h2 id="getting-started">Getting started</h2>
```

<!-- generated:options html.slug -->

Follows [rehype-slug](https://www.npmjs.com/package/rehype-slug).

| Option   | Type     | Default | Description         |
| -------- | -------- | ------- | ------------------- |
| `prefix` | `string` | `""`    | Put before each id. |

<!-- /generated -->

## html.highlight

Colors code blocks with TextMate grammars and themes, as Shiki does, natively.

````js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = textlane()
  .use(md.parse())
  .use(md.toHtml())
  .use(html.highlight({ theme: "github-light" }))
  .use(html.stringify())
  .processSync("```js\nlet a = 1\n```")
file.value
// <pre class="shiki github-light" style="background-color:#fff;color:#24292e"  …
````

<!-- generated:options html.highlight -->

Follows [@shikijs/rehype](https://www.npmjs.com/package/@shikijs/rehype).

| Option             | Type                              | Default | Description                                                                     |
| ------------------ | --------------------------------- | ------- | ------------------------------------------------------------------------------- |
| `themes`           | `Record<string, unknown> \| null` | `null`  | The theme of each color mode, `light` first, as `{"light": …, "dark": …}`.      |
| `theme`            | `string \| null`                  | `null`  | The one theme, when `themes` is not given.                                      |
| `defaultLanguage`  | `string \| null`                  | `null`  | The language of a block that names none; such a block is left alone without it. |
| `fallbackLanguage` | `string \| null`                  | `null`  | The language of a block naming one Shiki does not know; left alone without it.  |

<!-- /generated -->

## html.toMarkdown

Turns hast into mdast, so HTML can be written out as Markdown.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as md from "textlane/md"

const file = textlane()
  .use(html.parse({ fragment: true }))
  .use(html.toMarkdown())
  .use(md.stringify())
  .processSync("<h2>Title</h2><p>Some <strong>bold</strong> text.</p>")
file.value
// ## Title
//
// Some **bold** text.
```

<!-- generated:options html.toMarkdown -->

Follows [rehype-remark](https://www.npmjs.com/package/rehype-remark).

| Option      | Type               | Default | Description                                                                                                     |
| ----------- | ------------------ | ------- | --------------------------------------------------------------------------------------------------------------- |
| `document`  | `boolean`          | `false` | Wrap all phrasing content of the root in paragraphs, as for a whole document.                                   |
| `newlines`  | `boolean`          | `false` | Collapse whitespace holding a line ending to that line ending instead of a space.                               |
| `checked`   | `string \| null`   | `null`  | The text of a checked checkbox or radio, `[x]` by default.                                                      |
| `unchecked` | `string \| null`   | `null`  | The text of an unchecked checkbox or radio, `[ ]` by default.                                                   |
| `quotes`    | `string[] \| null` | `null`  | The quotes `<q>` uses, an opening and closing character each, alternating by how deep it nests; `"` by default. |

<!-- /generated -->

## html.toNaturalLanguage

Turns hast into nlcst, so `nl.*` plugins can check the prose of an HTML document.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as nl from "textlane/nl"

const file = textlane()
  .use(html.parse({ fragment: true }))
  .use(html.toNaturalLanguage())
  .use(nl.repeatedWords())
  .use(nl.stringify())
  .processSync("<p>It is the the best.</p>")
file.messages[0].reason
// Unexpected repeated `the`, remove one occurrence
```

<!-- generated:options html.toNaturalLanguage -->

Follows [rehype-retext](https://www.npmjs.com/package/rehype-retext).

No options.

<!-- /generated -->

## html.stringify

Serializes hast into HTML; its options pick quoting, closing and escaping.

```js
import { textlane } from "textlane"
import * as html from "textlane/html"

const file = textlane()
  .use(html.parse({ fragment: true }))
  .use(html.stringify({ quote: "'" }))
  .processSync('<img alt="Logo" src="logo.svg">')
file.value
// <img alt='Logo' src='logo.svg'>
```

<!-- generated:options html.stringify -->

Follows [rehype-stringify](https://www.npmjs.com/package/rehype-stringify).

| Option                     | Type               | Default | Description                                                                              |
| -------------------------- | ------------------ | ------- | ---------------------------------------------------------------------------------------- |
| `allowDangerousHtml`       | `boolean`          | `false` | Write raw nodes as they are instead of escaping them.                                    |
| `allowDangerousCharacters` | `boolean`          | `false` | Leave characters that are only dangerous in some contexts, such as `` ` ``, as they are. |
| `allowParseErrors`         | `boolean`          | `false` | Leave characters as they are where browsers accept them though the spec does not.        |
| `bogusComments`            | `boolean`          | `false` | Write comments as bogus comments, `<?a>`.                                                |
| `closeEmptyElements`       | `boolean`          | `false` | Close empty foreign elements, `<a />`.                                                   |
| `closeSelfClosing`         | `boolean`          | `false` | Close void elements, `<br />`.                                                           |
| `collapseEmptyAttributes`  | `boolean`          | `false` | Write empty attributes as bare names, `a` rather than `a=""`.                            |
| `omitOptionalTags`         | `boolean`          | `false` | Leave out tags the HTML spec lets a parser imply.                                        |
| `preferUnquoted`           | `boolean`          | `false` | Leave attribute values unquoted where that is safe.                                      |
| `quote`                    | `string \| null`   | `null`  | The quote attribute values are wrapped in: `"` or `'`.                                   |
| `quoteSmart`               | `boolean`          | `false` | Use the other quote when the value holds more of the preferred one.                      |
| `space`                    | `string \| null`   | `null`  | The space the tree is in: `html` or `svg`.                                               |
| `tightAttributes`          | `boolean`          | `false` | Leave out the space between quoted attributes.                                           |
| `tightCommaSeparatedLists` | `boolean`          | `false` | Join comma-separated values with `,` rather than `, `.                                   |
| `tightDoctype`             | `boolean`          | `false` | Write `<!doctypehtml>`.                                                                  |
| `tightSelfClosing`         | `boolean`          | `false` | Leave out the space before `/>` when the attributes allow.                               |
| `upperDoctype`             | `boolean`          | `false` | Write `<!DOCTYPE html>`.                                                                 |
| `voids`                    | `string[] \| null` | `null`  | The tag names of void elements, which have no closing tag.                               |

<!-- /generated -->
