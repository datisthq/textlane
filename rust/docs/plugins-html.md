---
title: HTML plugins
description: Every HTML plugin, its options and the unified package it follows.
path: /rust/html-plugins/
icon: code
order: 7
---

# HTML plugins

HTML plugins parse HTML into hast, transform it and turn it into HTML, Markdown trees or natural-language trees. A Markdown pipeline reaches them through `md.toHtml`.

The tables list each option as `Pipeline` takes it, by its JSON key and type; each plugin
struct's own fields and Rust types are on [docs.rs](https://docs.rs/textlane).

## html.parse

Parses HTML into a hast tree, as a browser does: a document by default, a fragment by option.

```rust
use textlane::{Processor, html};

let output = Processor::new()
    .with(html::Parse { fragment: true, ..Default::default() })
    .with(html::Stringify::default())
    .freeze()?
    .process("<p>One<p>Two")?;
output.value
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

```rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::ToHtml { allow_dangerous_html: true, ..Default::default() })
    .with(html::Raw::default())
    .with(html::Stringify::default())
    .freeze()?
    .process("<details><summary>More</summary>\n\n*Hidden*\n\n</details>")?;
output.value
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

```rust
use textlane::{Processor, html};

let output = Processor::new()
    .with(html::Parse { fragment: true, ..Default::default() })
    .with(html::Format::default())
    .with(html::Stringify::default())
    .freeze()?
    .process("<ul><li>One</li><li>Two</li></ul>")?;
output.value
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

```rust
use textlane::{Processor, html};

let output = Processor::new()
    .with(html::Parse { fragment: true, ..Default::default() })
    .with(html::Sanitize::default())
    .with(html::Stringify::default())
    .freeze()?
    .process("<a href=\"javascript:alert(1)\" onclick=\"x()\">Hi</a><script>x()</script>")?;
output.value
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

```rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::ToHtml::default())
    .with(html::Slug::default())
    .with(html::Stringify::default())
    .freeze()?
    .process("## Getting started")?;
output.value
// <h2 id="getting-started">Getting started</h2>
```

<!-- generated:options html.slug -->

Follows [rehype-slug](https://www.npmjs.com/package/rehype-slug).

| Option   | Type     | Default | Description         |
| -------- | -------- | ------- | ------------------- |
| `prefix` | `string` | `""`    | Put before each id. |

<!-- /generated -->

## html.externalLinks

Marks links to other sites as rehype-external-links does: `rel` (`nofollow` by default), `target`, more `properties`, and `content` appended in a `<span>`. A link counts as external when its URL is absolute in one of `protocols` (`http` and `https` by default) or protocol-relative.

```rust
use textlane::{Processor, html, md};

let links = html::ExternalLinks {
    target: Some("_blank".into()),
    rel: Some(html::Tokens::Many(vec!["noopener".into()])),
    ..Default::default()
};
let output = Processor::new()
    .with(md::Parse::default())
    .with(md::ToHtml::default())
    .with(links)
    .with(html::Stringify::default())
    .freeze()?
    .process("See [npm](https://www.npmjs.com) or the [guide](/guide).")?;
output.value
// <p>See <a href="https://www.npmjs.com" rel="noopener" target="_blank">npm</a> or the <a href="/guide">guide</a>.</p>
```

<!-- generated:options html.externalLinks -->

Follows [rehype-external-links](https://www.npmjs.com/package/rehype-external-links).

| Option              | Type                              | Default | Description                                                                                                                             |
| ------------------- | --------------------------------- | ------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `target`            | `string \| null`                  | `null`  | The `target` of each external link, such as `_blank`.                                                                                   |
| `rel`               | `Tokens \| null`                  |         | The `rel` of each external link: a list, or tokens separated by white space; `nofollow` when left out or empty, none for an empty list. |
| `protocols`         | `string[] \| null`                | `null`  | The protocols of absolute URLs that count as external; `http` and `https` when left out. A protocol-relative `//…` URL always counts.   |
| `properties`        | `Record<string, unknown> \| null` | `null`  | More properties for each external link, as hast names them.                                                                             |
| `content`           | `unknown`                         | `null`  | hast content appended to each external link in a `<span>`: one node or a list.                                                          |
| `contentProperties` | `Record<string, unknown> \| null` | `null`  | The properties of that `<span>`.                                                                                                        |

**`Tokens`**

Tokens: a list, or one string of them separated by white space.

Takes `string | string[]`.

<!-- /generated -->

## html.unwrapImages

Replaces each paragraph that holds only images, links around them and white space with its content, as rehype-unwrap-images does, so images are not wrapped in `<p>`.

```rust
use textlane::{Processor, html};

let output = Processor::new()
    .with(html::Parse { fragment: true, ..Default::default() })
    .with(html::UnwrapImages)
    .with(html::Stringify::default())
    .freeze()?
    .process(r#"<p><img src="a.png"></p>"#)?;
output.value
// <img src="a.png">
```

<!-- generated:options html.unwrapImages -->

Follows [rehype-unwrap-images](https://www.npmjs.com/package/rehype-unwrap-images).

No options.

<!-- /generated -->

## html.extractToc

Stores the headings, nested by rank, as the file's `data.toc`, as @stefanprobst/rehype-extract-toc does: each `{ depth, value, id?, children? }`. Run it after `html.slug` so headings carry their ids.

```rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::ToHtml::default())
    .with(html::Slug::default())
    .with(html::ExtractToc)
    .with(html::Stringify::default())
    .freeze()?
    .process("# Guide\n\n## Install")?;
output.file.data["toc"]
// [{ depth: 1, value: "Guide", id: "guide", children: [{ depth: 2, value: "Install", id: "install" }] }]
```

<!-- generated:options html.extractToc -->

Follows [@stefanprobst/rehype-extract-toc](https://www.npmjs.com/package/@stefanprobst/rehype-extract-toc).

No options.

<!-- /generated -->

## html.highlight

Colors code blocks with TextMate grammars and themes, as Shiki does, natively.

````rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::ToHtml::default())
    .with(html::Highlight { theme: Some("github-light".into()), ..Default::default() })
    .with(html::Stringify::default())
    .freeze()?
    .process("```js\nlet a = 1\n```")?;
output.value
// <pre class="shiki github-light" style="background-color:#fff;color:#24292e"  …
````

`transformers` runs @shikijs/transformers natively, by name or with their options:
`NotationDiff`, `NotationFocus`, `NotationHighlight` and `NotationErrorLevel` read
`// [!code ++]`-style comments, `MetaHighlight` reads `{1,3-5}` after the fence's language, `MetaWordHighlight` marks every `/word/` it names
and `RenderLineNumber` numbers lines, each marking lines with Shiki's classes.
`inline: Some(html::InlineCode::TailingCurlyColon)` colors inline code written
`` `a(){:js}` ``, `color_replacements` and `add_language_class` are Shiki's own, and
`meta_property` puts a fence's meta on its `<pre>`.

```rust
use textlane::html::{Highlight, HighlightTransformer, TransformerName};

let highlight = Highlight {
    theme: Some("github-light".into()),
    transformers: vec![HighlightTransformer::Name(TransformerName::NotationDiff)],
    ..Default::default()
};
```

<!-- generated:options html.highlight -->

Follows [@shikijs/rehype](https://www.npmjs.com/package/@shikijs/rehype).

| Option              | Type                               | Default | Description                                                                                                                                      |
| ------------------- | ---------------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `themes`            | `Record<string, unknown> \| null`  | `null`  | The theme of each color mode, `light` first, as `{"light": …, "dark": …}`.                                                                       |
| `theme`             | `string \| null`                   | `null`  | The one theme, when `themes` is not given.                                                                                                       |
| `defaultLanguage`   | `string \| null`                   | `null`  | The language of a block that names none; such a block is left alone without it.                                                                  |
| `fallbackLanguage`  | `string \| null`                   | `null`  | The language of a block naming one Shiki does not know; left alone without it.                                                                   |
| `colorReplacements` | `Record<string, ColorReplacement>` |         | Colors to replace, by their lowercase spelling: a color for every theme, or a theme's name to a map of its own.                                  |
| `addLanguageClass`  | `boolean`                          | `false` | Add `language-<name>` to each highlighted `<code>`.                                                                                              |
| `inline`            | `InlineCode \| null`               |         | Highlight inline code too, by how it names its language: `tailing-curly-colon` reads `` `a(){:js}` ``.                                           |
| `metaProperty`      | `string \| null`                   | `null`  | The `<pre>` property that gets a block's meta, the text after its language; the JSON stand-in for `parseMetaString: meta => ({ [name]: meta })`. |
| `transformers`      | `HighlightTransformer[]`           |         | The transformers of @shikijs/transformers to run, in order, by name or with options.                                                             |

**`ColorReplacement`**

A replacement color, or a theme's own replacements.

Takes `string | Record<string, string>`.

**`InlineCode`**

How inline code names its language.

Takes `"tailing-curly-colon"`.

**`HighlightTransformer`**

A transformer of @shikijs/transformers: its name alone, or its name and options.

Takes `TransformerName | ConfiguredTransformer`.

**`TransformerName`**

The transformers `html.highlight` runs natively.

Takes `"notationDiff" | "notationFocus" | "notationHighlight" | "notationErrorLevel" | "metaHighlight" | "metaWordHighlight" | "renderLineNumber"`.

**`ConfiguredTransformer`**

A transformer with its options, as @shikijs/transformers names them.

Takes `{ name: "notationDiff"; classLineAdd?: Classes | null; classLineRemove?: Classes | null; classActivePre?: Classes | null; classActiveCode?: Classes | null } | { name: "notationFocus"; classActiveLine?: Classes | null; classActivePre?: Classes | null; classActiveCode?: Classes | null } | { name: "notationHighlight"; classActiveLine?: Classes | null; classActivePre?: Classes | null; classActiveCode?: Classes | null } | { name: "notationErrorLevel"; classMap?: Record<string, Classes> | null; classActivePre?: Classes | null; classActiveCode?: Classes | null } | { name: "metaHighlight"; className?: Classes | null; zeroIndexed?: boolean } | { name: "metaWordHighlight"; className?: Classes | null } | { name: "renderLineNumber"; classLineNumber?: Classes | null; start?: number | null }`.

**`Classes`**

Class names: a list, or one string of them separated by white space.

Takes `string | string[]`.

<!-- /generated -->

## html.katex

Renders math with KaTeX, as rehype-katex does, natively: each `$…$`, `$$…$$` and ` ```math ` that `md.math` and `md.toHtml` leave becomes KaTeX's HTML and MathML. Pages need KaTeX's stylesheet and fonts. A formula that fails to render is reported in the file's messages and shown in `errorColor`.

```rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Math::default())
    .with(md::ToHtml::default())
    .with(html::Katex::default())
    .with(html::Stringify::default())
    .freeze()?
    .process("Euler: $e^{i\\pi} = -1$")?;
output.value
// <p>Euler: <span class="katex"><span class="katex-mathml"><math …
```

<!-- generated:options html.katex -->

Follows [rehype-katex](https://www.npmjs.com/package/rehype-katex).

| Option             | Type                     | Default | Description                                                                                                                                        |
| ------------------ | ------------------------ | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| `output`           | `KatexOutput \| null`    |         | The markup to write, both HTML and MathML by default.                                                                                              |
| `leqno`            | `boolean`                | `false` | Put equation numbers on the left.                                                                                                                  |
| `fleqn`            | `boolean`                | `false` | Align display math to the left instead of centering it.                                                                                            |
| `errorColor`       | `string \| null`         | `null`  | The color of math that fails to render, `#cc0000` by default.                                                                                      |
| `macros`           | `Record<string, string>` | `{}`    | Macros by name, each to the TeX it expands to, as `{"\\RR": "\\mathbb{R}"}`.                                                                       |
| `minRuleThickness` | `number \| null`         | `null`  | The thinnest a rule, such as a fraction line, is drawn, in ems.                                                                                    |
| `colorIsTextColor` | `boolean`                | `false` | Let `\color` color the text after it, as LaTeX does.                                                                                               |
| `strict`           | `KatexStrict \| null`    |         | What to do with input LaTeX would reject: `true` or `"error"` fails it, `"warn"` prints a warning, `false` or `"ignore"` (the default) accepts it. |
| `trust`            | `boolean`                | `false` | Allow commands that can run untrusted input, such as `\href` and `\includegraphics`.                                                               |
| `maxSize`          | `number \| null`         | `null`  | The largest a user-given size may be, in ems.                                                                                                      |
| `maxExpand`        | `number \| null`         | `null`  | How many macro expansions one formula may take, 1000 by default.                                                                                   |
| `globalGroup`      | `boolean`                | `false` | Let definitions such as `\def` last past the formula they are in.                                                                                  |

**`KatexOutput`**

The markup KaTeX writes.

Takes `"html" | "mathml" | "htmlAndMathml"`.

**`KatexStrict`**

How strictly KaTeX holds input to LaTeX.

Takes `boolean | KatexStrictMode`.

**`KatexStrictMode`**

One level of KaTeX's strictness.

Takes `"ignore" | "warn" | "error"`.

<!-- /generated -->

## html.toMarkdown

Turns hast into mdast, so HTML can be written out as Markdown.

```rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(html::Parse { fragment: true, ..Default::default() })
    .with(html::ToMarkdown::default())
    .with(md::Stringify::default())
    .freeze()?
    .process("<h2>Title</h2><p>Some <strong>bold</strong> text.</p>")?;
output.value
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

```rust
use textlane::{Processor, html, nl};

let output = Processor::new()
    .with(html::Parse { fragment: true, ..Default::default() })
    .with(html::ToNaturalLanguage)
    .with(nl::RepeatedWords)
    .with(nl::Stringify)
    .freeze()?
    .process("<p>It is the the best.</p>")?;
output.file.messages[0].reason
// Unexpected repeated `the`, remove one occurrence
```

<!-- generated:options html.toNaturalLanguage -->

Follows [rehype-retext](https://www.npmjs.com/package/rehype-retext).

No options.

<!-- /generated -->

## html.stringify

Serializes hast into HTML; its options pick quoting, closing and escaping.

```rust
use textlane::{Processor, html};

let output = Processor::new()
    .with(html::Parse { fragment: true, ..Default::default() })
    .with(html::Stringify { quote: Some("'".into()), ..Default::default() })
    .freeze()?
    .process("<img alt=\"Logo\" src=\"logo.svg\">")?;
output.value
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
