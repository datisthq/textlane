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

`minify: Some(Minify::Enabled(true))` only collapses whitespace, as rehype-minify-whitespace does, adding no line endings or indentation; `Minify::Options(MinifyOptions { newlines: true })` keeps a line ending where a collapsed run held one.

<!-- generated:options html.format -->

Follows [rehype-format](https://www.npmjs.com/package/rehype-format).

| Option          | Type              | Default | Description                                                                                                                                                          |
| --------------- | ----------------- | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `blanks`        | `string[]`        | `[]`    | Tag names of elements to put a blank line between, when both are one.                                                                                                |
| `indent`        | `Indent \| null`  |         | One level of indentation: a number of spaces or the text itself; two spaces when left out.                                                                           |
| `indentInitial` | `boolean \| null` | `null`  | Indent the first level, usually `<html>`'s `head` and `body`; `true` when left out.                                                                                  |
| `minify`        | `Minify \| null`  |         | Only collapse whitespace, as rehype-minify-whitespace does, adding no line endings or indentation; `{ newlines: true }` collapses a run holding a line ending to it. |

**`Indent`**

One level of indentation.

Takes `number | string`.

**`Minify`**

`html.format`'s `minify`: on, or on with rehype-minify-whitespace's options.

Takes `boolean | MinifyOptions`.

**`MinifyOptions`**

rehype-minify-whitespace's options.

| Option     | Type      | Default | Description                                                                       |
| ---------- | --------- | ------- | --------------------------------------------------------------------------------- |
| `newlines` | `boolean` | `false` | Collapse a run of whitespace holding a line ending to a line ending, not a space. |

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

`autolink: Some(Autolink::Enabled(true))` also links each heading to itself, as rehype-autolink-headings does after rehype-slug: an `<a href="#id">` prepended by default, or placed by `behavior` (`Append`, `Wrap`, `Before`, `After`). Its `content` and `group` are hast as JSON.

<!-- generated:options html.slug -->

Follows [rehype-slug](https://www.npmjs.com/package/rehype-slug).

| Option            | Type               | Default | Description                                                                                              |
| ----------------- | ------------------ | ------- | -------------------------------------------------------------------------------------------------------- |
| `prefix`          | `string`           | `""`    | Put before each id.                                                                                      |
| `reserveExisting` | `boolean`          | `false` | Count the ids already in the document as taken, so a generated id never repeats one a link may point at. |
| `ranks`           | `number[] \| null` | `null`  | The heading ranks, 1 to 6, that get ids; every heading when left out.                                    |
| `autolink`        | `Autolink \| null` |         | Link each heading with an `id` to itself, as rehype-autolink-headings does: `true`, or its options.      |

**`Autolink`**

`html.slug`'s `autolink`: on, or on with rehype-autolink-headings' options.

Takes `boolean | AutolinkOptions`.

**`AutolinkOptions`**

rehype-autolink-headings' options, JSON values only.

| Option              | Type                              | Default | Description                                                                                                                                                |
| ------------------- | --------------------------------- | ------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `behavior`          | `AutolinkBehavior`                |         | Where the link goes: in the heading first or last, around its content, or before or after it.                                                              |
| `properties`        | `Record<string, unknown> \| null` | `null`  | The link's properties, as hast names them; `ariaHidden` and `tabIndex: -1` for `prepend` and `append` when left out.                                       |
| `content`           | `unknown`                         | `null`  | hast content of the link, one node or a list; `<span class="icon icon-link">` when left out, except for `wrap`, which appends it to the heading's content. |
| `group`             | `unknown`                         | `null`  | A hast element wrapping the heading and its link, for `before` and `after`.                                                                                |
| `headingProperties` | `Record<string, unknown> \| null` | `null`  | Properties set on each linked heading.                                                                                                                     |
| `test`              | `TagNames \| null`                |         | The tag names of the headings to link; every heading when left out.                                                                                        |

**`AutolinkBehavior`**

Where `autolink` puts each heading's link.

Takes `"prepend" | "append" | "wrap" | "before" | "after"`.

**`TagNames`**

One tag name or several.

Takes `string | string[]`.

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

## html.math

Turns `$…$` and `$$…$$` in text into the `<code class="language-math math-inline">` and `math-display` elements `md.math` makes, for `html.katex` to render. It reads dollars by pandoc's rule, so prices stay text: an opening `$` is followed by a non-space, a closing one follows a non-space and is not followed by a digit; `\$` is a dollar sign. Links, code, scripts, form text, `math` and `svg` are left alone, or the elements `ignore` names.

```rust
use textlane::{Processor, html};

let output = Processor::new()
    .with(html::Parse { fragment: true, ..Default::default() })
    .with(html::Math::default())
    .with(html::Stringify::default())
    .freeze()?
    .process(r#"<p>Area $\pi r^2$, price $5</p>"#)?;
output.value
// <p>Area <code class="language-math math-inline">\pi r^2</code>, price $5</p>
```

<!-- generated:options html.math -->

| Option                 | Type               | Default | Description                                                                                                       |
| ---------------------- | ------------------ | ------- | ----------------------------------------------------------------------------------------------------------------- |
| `singleDollarTextMath` | `boolean`          | `true`  | Let one dollar, `$a$`, form math, as well as two.                                                                 |
| `ignore`               | `string[] \| null` | `null`  | Elements, by tag name, whose text is left alone; links, code, scripts, form text, `math` and `svg` when left out. |
| `ignoreAttributes`     | `string[]`         | `[]`    | Attributes, such as `data-literal`, whose elements' text is left alone.                                           |

<!-- /generated -->

## html.linkify

Turns bare URLs and email addresses in text into links, as GitHub's literal autolinks do: a URL with `://` as written, a `www.` one under `https://`, an email under `mailto:`. A bare domain, such as `index.html`, stays text. Links, code, scripts, form text, `math` and `svg` are left alone, or the elements `ignore` names, and any element with an attribute `ignoreAttributes` names.

```rust
use textlane::{Processor, html};

let output = Processor::new()
    .with(html::Parse { fragment: true, ..Default::default() })
    .with(html::Linkify::default())
    .with(html::Stringify::default())
    .freeze()?
    .process(r#"<p>See www.example.com or a@example.com</p>"#)?;
output.value
// <p>See <a href="https://www.example.com">www.example.com</a> or <a href="mailto:a@example.com">a@example.com</a></p>
```

<!-- generated:options html.linkify -->

| Option             | Type               | Default | Description                                                                                                       |
| ------------------ | ------------------ | ------- | ----------------------------------------------------------------------------------------------------------------- |
| `ignore`           | `string[] \| null` | `null`  | Elements, by tag name, whose text is left alone; links, code, scripts, form text, `math` and `svg` when left out. |
| `ignoreAttributes` | `string[]`         | `[]`    | Attributes, such as `data-literal`, whose elements' text is left alone.                                           |

<!-- /generated -->

## html.gemoji

Turns gemoji shortcodes in text, `:rocket:`, into their emoji, as `md.gemoji` does in Markdown; with `accessible`, each sits in a `<span role="img">` labelled with its name. Text that only looks like one, `10:30:00`, stays as written.

```rust
use textlane::{Processor, html};

let output = Processor::new()
    .with(html::Parse { fragment: true, ..Default::default() })
    .with(html::Gemoji::default())
    .with(html::Stringify::default())
    .freeze()?
    .process(r#"<p>Ship it :rocket:</p>"#)?;
output.value
// <p>Ship it 🚀</p>
```

<!-- generated:options html.gemoji -->

| Option             | Type               | Default | Description                                                                                                       |
| ------------------ | ------------------ | ------- | ----------------------------------------------------------------------------------------------------------------- |
| `accessible`       | `boolean`          | `false` | Wrap each emoji in `<span role="img" aria-label="…">`, labelled with its name.                                    |
| `ignore`           | `string[] \| null` | `null`  | Elements, by tag name, whose text is left alone; links, code, scripts, form text, `math` and `svg` when left out. |
| `ignoreAttributes` | `string[]`         | `[]`    | Attributes, such as `data-literal`, whose elements' text is left alone.                                           |

<!-- /generated -->

## html.pathRewrite

Rewrites the URLs in `href`, `src`, `srcset` and `poster`, or the attributes `attributes` names, as a site serves them, as `md.pathRewrite` does for Markdown: a relative URL is resolved against the file's path and made root-relative, every root-relative one goes under `base` once, and an `href` naming a file in `pages` takes that page's URL.

```rust
use textlane::{Processor, html};

let output = Processor::new()
    .with(html::Parse { fragment: true, ..Default::default() })
    .with(html::PathRewrite { base: Some("/docs".into()), ..Default::default() })
    .with(html::Stringify::default())
    .freeze()?
    .process(r#"<a href="/guide.html">Guide</a>"#)?;
output.value
// <a href="/docs/guide.html">Guide</a>
```

<!-- generated:options html.pathRewrite -->

| Option       | Type                     | Default | Description                                                                                                              |
| ------------ | ------------------------ | ------- | ------------------------------------------------------------------------------------------------------------------------ |
| `root`       | `string \| null`         | `null`  | The directory URLs count from, as the file's path spells it; the path's own starting point when left out.                |
| `base`       | `string \| null`         | `null`  | The URL the site is served at, such as `/docs`, put before every root-relative URL once.                                 |
| `pages`      | `Record<string, string>` | `{}`    | Files, relative to `root`, to the URL of their page, such as `{"guide.html": "/guide/"}`; an `href` naming one takes it. |
| `attributes` | `string[] \| null`       | `null`  | The attributes whose URLs are rewritten; `href`, `src`, `srcset` and `poster` when left out.                             |

<!-- /generated -->

## html.extractToc

Stores the headings, nested by rank, as the file's `data.toc`, as @stefanprobst/rehype-extract-toc does: each `{ depth, value, id?, children? }`. Run it after `html.slug` so headings carry their ids.

```rust
use textlane::{Processor, html, md};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::ToHtml::default())
    .with(html::Slug::default())
    .with(html::ExtractToc::default())
    .with(html::Stringify::default())
    .freeze()?
    .process("# Guide\n\n## Install")?;
output.file.data["toc"]
// [{ depth: 1, value: "Guide", id: "guide", children: [{ depth: 2, value: "Install", id: "install" }] }]
```

`skip` lists attributes that keep a heading out, such as `data-toc-hidden`; what would nest under it nests under the heading before. A `hidden` heading stays listed, which suits a heading meant for the table of contents alone.

<!-- generated:options html.extractToc -->

Follows [@stefanprobst/rehype-extract-toc](https://www.npmjs.com/package/@stefanprobst/rehype-extract-toc).

| Option       | Type             | Default | Description                                                                                                                                                                                                       |
| ------------ | ---------------- | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `skip`       | `string[]`       | `[]`    | Attributes, such as `data-toc-hidden`, that keep a heading out of the table of contents, or `name=value`, such as `data-toc=false`, for one value of it; what would nest under it nests under the heading before. |
| `skipInside` | `string[]`       | `[]`    | Elements, by tag name, whose headings stay out, such as a card's title.                                                                                                                                           |
| `minDepth`   | `number \| null` | `null`  | The shallowest heading rank listed, 1 to 6.                                                                                                                                                                       |
| `maxDepth`   | `number \| null` | `null`  | The deepest heading rank listed, 1 to 6.                                                                                                                                                                          |

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

A grammar's time grows with the square of a line's length, so a line of 2,000 characters or more, typically minified code, is left one plain token, as Shiki's `tokenize_max_line_length` leaves it; Shiki has no such limit by default, and `0` lifts it.

<!-- generated:options html.highlight -->

Follows [@shikijs/rehype](https://www.npmjs.com/package/@shikijs/rehype).

| Option                  | Type                               | Default | Description                                                                                                                                                                                                                          |
| ----------------------- | ---------------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `themes`                | `Record<string, unknown> \| null`  | `null`  | The theme of each color mode, `light` first, as `{"light": …, "dark": …}`.                                                                                                                                                           |
| `theme`                 | `string \| null`                   | `null`  | The one theme, when `themes` is not given.                                                                                                                                                                                           |
| `defaultLanguage`       | `string \| null`                   | `null`  | The language of a block that names none; such a block is left alone without it.                                                                                                                                                      |
| `fallbackLanguage`      | `string \| null`                   | `null`  | The language of a block naming one Shiki does not know; left alone without it.                                                                                                                                                       |
| `colorReplacements`     | `Record<string, ColorReplacement>` |         | Colors to replace, by their lowercase spelling: a color for every theme, or a theme's name to a map of its own.                                                                                                                      |
| `addLanguageClass`      | `boolean`                          | `false` | Add `language-<name>` to each highlighted `<code>`.                                                                                                                                                                                  |
| `inline`                | `InlineCode \| null`               |         | Highlight inline code too, by how it names its language: `tailing-curly-colon` reads `` `a(){:js}` ``.                                                                                                                               |
| `metaProperty`          | `string \| null`                   | `null`  | The `<pre>` property that gets a block's meta, the text after its language; the JSON stand-in for `parseMetaString: meta => ({ [name]: meta })`.                                                                                     |
| `transformers`          | `HighlightTransformer[]`           |         | The transformers of @shikijs/transformers to run, in order, by name or with options.                                                                                                                                                 |
| `tokenizeMaxLineLength` | `number \| null`                   | `null`  | Lines at least this long, in UTF-16 units, are left one plain token as Shiki's own option leaves them; 2,000 when left out, where Shiki has no limit, since a grammar's time grows with the square of a line's length. `0` lifts it. |
| `lineAttributes`        | `Record<string, LineMark>`         |         | Attributes of a block's `<code>` or `<pre>` listing lines, `2,4-5`, each to the classes those lines get, or to `{line, pre}`, the second for the `<pre>`: `{"data-focus": {"line": "focused", "pre": "has-focused"}}`.               |
| `wordAttribute`         | `string \| null`                   | `null`  | The attribute of a block's `<code>` or `<pre>` listing words, separated by commas, marked at every place as `metaWordHighlight` marks a meta's `/word/`.                                                                             |
| `lineNumberAttribute`   | `string \| null`                   | `null`  | The attribute of a block's `<code>` or `<pre>` that numbers its lines as `renderLineNumber` does, from its value, or 1 when it is empty.                                                                                             |
| `skipElements`          | `boolean`                          | `false` | Leave a block alone when its code holds elements, such as `<mark>`, which highlighting would flatten into text.                                                                                                                      |
| `keepProperties`        | `boolean`                          | `false` | Keep the properties of a block's `<pre>` and `<code>`, such as `data-title`, on the ones Shiki makes, merging classes; Shiki's own win where both set one.                                                                           |

**`ColorReplacement`**

A replacement color, or a theme's own replacements.

Takes `string | Record<string, string>`.

**`InlineCode`**

How inline code names its language.

Takes `"tailing-curly-colon" | "class"`.

**`HighlightTransformer`**

A transformer of @shikijs/transformers: its name alone, or its name and options.

Takes `TransformerName | ConfiguredTransformer`.

**`LineMark`**

The classes the lines a `lineAttributes` attribute lists get, and maybe its `<pre>`.

Takes `Classes | { line: Classes; pre?: Classes | null }`.

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

| Option             | Type                     | Default | Description                                                                                                                                                   |
| ------------------ | ------------------------ | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `output`           | `KatexOutput \| null`    |         | The markup to write, both HTML and MathML by default.                                                                                                         |
| `leqno`            | `boolean`                | `false` | Put equation numbers on the left.                                                                                                                             |
| `fleqn`            | `boolean`                | `false` | Align display math to the left instead of centering it.                                                                                                       |
| `errorColor`       | `string \| null`         | `null`  | The color of math that fails to render, `#cc0000` by default.                                                                                                 |
| `macros`           | `Record<string, string>` | `{}`    | Macros by name, each to the TeX it expands to, as `{"\\RR": "\\mathbb{R}"}`.                                                                                  |
| `minRuleThickness` | `number \| null`         | `null`  | The thinnest a rule, such as a fraction line, is drawn, in ems.                                                                                               |
| `colorIsTextColor` | `boolean`                | `false` | Let `\color` color the text after it, as LaTeX does.                                                                                                          |
| `strict`           | `KatexStrict \| null`    |         | What to do with input LaTeX would reject: `true` or `"error"` fails it, `"warn"` prints a warning, `false` or `"ignore"` (the default) accepts it.            |
| `trust`            | `boolean`                | `false` | Allow commands that can run untrusted input, such as `\href` and `\includegraphics`.                                                                          |
| `maxSize`          | `number \| null`         | `null`  | The largest a user-given size may be, in ems.                                                                                                                 |
| `maxExpand`        | `number \| null`         | `null`  | How many macro expansions one formula may take, 1000 by default.                                                                                              |
| `globalGroup`      | `boolean`                | `false` | Let definitions such as `\def` last past the formula they are in.                                                                                             |
| `tagNames`         | `string[]`               | `[]`    | Elements, by tag name, that hold math besides the classed ones, such as `my-math`; one is display math when it has a `display` attribute other than `inline`. |

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

| Option      | Type                     | Default | Description                                                                                                                                                                                                 |
| ----------- | ------------------------ | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `document`  | `boolean`                | `false` | Wrap all phrasing content of the root in paragraphs, as for a whole document.                                                                                                                               |
| `newlines`  | `boolean`                | `false` | Collapse whitespace holding a line ending to that line ending instead of a space.                                                                                                                           |
| `checked`   | `string \| null`         | `null`  | The text of a checked checkbox or radio, `[x]` by default.                                                                                                                                                  |
| `unchecked` | `string \| null`         | `null`  | The text of an unchecked checkbox or radio, `[ ]` by default.                                                                                                                                               |
| `quotes`    | `string[] \| null`       | `null`  | The quotes `<q>` uses, an opening and closing character each, alternating by how deep it nests; `"` by default.                                                                                             |
| `math`      | `boolean`                | `false` | Turn MathML `<math>` holding a TeX annotation, as `html.katex` writes it, into math nodes, `$…$` once `md.math` writes them, display math for `display="block"`, and drop KaTeX's visual `katex-html` copy. |
| `codeMeta`  | `Record<string, string>` | `{}`    | Attributes of a code block's `<pre>` or `<code>` written into its meta, each to the key it takes there: `{"data-title": "title"}` makes `title="a.js"`.                                                     |

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
