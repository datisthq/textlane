---
title: HTML plugins
description: Every HTML plugin, its options and the unified package it follows.
path: /python/html-plugins/
icon: code
order: 7
---

# HTML plugins

HTML plugins parse HTML into hast, transform it and turn it into HTML, Markdown trees or natural-language trees. A Markdown pipeline reaches them through `md.toHtml`.

## html.parse

Parses HTML into a hast tree, as a browser does: a document by default, a fragment by option.

```python
from textlane import html, textlane

file = (
    textlane()
    .use(html.parse({"fragment": True}))
    .use(html.stringify())
    .process("<p>One<p>Two")
)
file["value"]
# <p>One</p><p>Two</p>
```

<!-- generated:options html.parse -->

Follows [rehype-parse](https://www.npmjs.com/package/rehype-parse).

| Option            | Type   | Default | Description                                                               |
| ----------------- | ------ | ------- | ------------------------------------------------------------------------- |
| `fragment`        | `bool` | `False` | Parse a fragment, as in a `template`, rather than a whole document.       |
| `emitParseErrors` | `bool` | `False` | Report parse errors as messages on the file.                              |
| `verbose`         | `bool` | `False` | Add the positions of opening and closing tags and attributes to elements. |

<!-- /generated -->

## html.raw

Reparses the raw HTML a Markdown document carries, so it becomes real hast nodes.

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.to_html({"allowDangerousHtml": True}))
    .use(html.raw())
    .use(html.stringify())
    .process("<details><summary>More</summary>\n\n*Hidden*\n\n</details>")
)
file["value"]
# <details><summary>More</summary>
# <p><em>Hidden</em></p>
# </details>
```

<!-- generated:options html.raw -->

Follows [rehype-raw](https://www.npmjs.com/package/rehype-raw).

| Option        | Type        | Default | Description                                                                                         |
| ------------- | ----------- | ------- | --------------------------------------------------------------------------------------------------- |
| `passThrough` | `List[str]` | `[]`    | Node types, unknown to hast, to copy through as they are; their children are reparsed on their own. |
| `tagfilter`   | `bool`      | `False` | Escape the opening `<` of the tags GFM's tagfilter disallows in raw nodes.                          |

<!-- /generated -->

## html.format

Indents the HTML and collapses whitespace, for output a person reads.

```python
from textlane import html, textlane

file = (
    textlane()
    .use(html.parse({"fragment": True}))
    .use(html.format())
    .use(html.stringify())
    .process("<ul><li>One</li><li>Two</li></ul>")
)
file["value"]
# <ul>
#   <li>One</li>
#   <li>Two</li>
# </ul>
```

`"minify": True` only collapses whitespace, as rehype-minify-whitespace does, adding no line endings or indentation; `"minify": {"newlines": True}` keeps a line ending where a collapsed run held one.

<!-- generated:options html.format -->

Follows [rehype-format](https://www.npmjs.com/package/rehype-format).

| Option          | Type               | Default | Description                                                                                                                                                          |
| --------------- | ------------------ | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `blanks`        | `List[str]`        | `[]`    | Tag names of elements to put a blank line between, when both are one.                                                                                                |
| `indent`        | `Optional[Indent]` |         | One level of indentation: a number of spaces or the text itself; two spaces when left out.                                                                           |
| `indentInitial` | `Optional[bool]`   | `None`  | Indent the first level, usually `<html>`'s `head` and `body`; `true` when left out.                                                                                  |
| `minify`        | `Optional[Minify]` |         | Only collapse whitespace, as rehype-minify-whitespace does, adding no line endings or indentation; `{ newlines: true }` collapses a run holding a line ending to it. |

**`Indent`**

One level of indentation.

Takes `Union[int, str]`.

**`Minify`**

`html.format`'s `minify`: on, or on with rehype-minify-whitespace's options.

Takes `Union[bool, MinifyOptions]`.

**`MinifyOptions`**

rehype-minify-whitespace's options.

| Option     | Type   | Default | Description                                                                       |
| ---------- | ------ | ------- | --------------------------------------------------------------------------------- |
| `newlines` | `bool` | `False` | Collapse a run of whitespace holding a line ending to a line ending, not a space. |

<!-- /generated -->

## html.sanitize

Keeps only the elements and attributes a schema allows — GitHub's, by default — so untrusted HTML is safe to show.

```python
from textlane import html, textlane

file = (
    textlane()
    .use(html.parse({"fragment": True}))
    .use(html.sanitize())
    .use(html.stringify())
    .process("<a href=\"javascript:alert(1)\" onclick=\"x()\">Hi</a><script>x()</script>")
)
file["value"]
# <a>Hi</a>
```

<!-- generated:options html.sanitize -->

Follows [rehype-sanitize](https://www.npmjs.com/package/rehype-sanitize).

| Option          | Type                                             | Default | Description                                                                                                 |
| --------------- | ------------------------------------------------ | ------- | ----------------------------------------------------------------------------------------------------------- |
| `allowComments` | `Optional[bool]`                                 | `None`  | Keep comments.                                                                                              |
| `allowDoctypes` | `Optional[bool]`                                 | `None`  | Keep doctypes.                                                                                              |
| `ancestors`     | `Optional[Dict[str, List[str]]]`                 | `None`  | Elements allowed only inside one of these ancestors, by tag name.                                           |
| `attributes`    | `Optional[Dict[str, List[AttributeDefinition]]]` |         | Properties allowed per tag name, `*` for every element; a list names a property and the values it may have. |
| `clobber`       | `Optional[List[str]]`                            | `None`  | Properties whose values get `clobber_prefix`, so they cannot clobber globals.                               |
| `clobberPrefix` | `Optional[str]`                                  | `None`  | The prefix of clobbered values.                                                                             |
| `protocols`     | `Optional[Dict[str, List[str]]]`                 | `None`  | URL properties and the protocols they may use.                                                              |
| `required`      | `Optional[Dict[str, Dict[str, Primitive]]]`      |         | Properties set on elements whatever they had, by tag name.                                                  |
| `strip`         | `Optional[List[str]]`                            | `None`  | Elements dropped with their content rather than unwrapped.                                                  |
| `tagNames`      | `Optional[List[str]]`                            | `None`  | The elements kept.                                                                                          |

**`AttributeDefinition`**

A property a sanitize schema allows: by name, or a name and its allowed values.

Takes `Union[str, List[Primitive]]`.

**`Primitive`**

A primitive property value.

Takes `Union[bool, float, str]`.

<!-- /generated -->

## html.slug

Gives each heading an `id` from its text, as GitHub does, so it can be linked to.

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.to_html())
    .use(html.slug())
    .use(html.stringify())
    .process("## Getting started")
)
file["value"]
# <h2 id="getting-started">Getting started</h2>
```

`"autolink": True` also links each heading to itself, as rehype-autolink-headings does after rehype-slug: an `<a href="#id">` prepended by default, or placed by `behavior` (`append`, `wrap`, `before`, `after`). Its options take JSON values only: `content` and `group` are hast nodes, not functions.

<!-- generated:options html.slug -->

Follows [rehype-slug](https://www.npmjs.com/package/rehype-slug).

| Option            | Type                  | Default | Description                                                                                              |
| ----------------- | --------------------- | ------- | -------------------------------------------------------------------------------------------------------- |
| `prefix`          | `str`                 | `""`    | Put before each id.                                                                                      |
| `reserveExisting` | `bool`                | `False` | Count the ids already in the document as taken, so a generated id never repeats one a link may point at. |
| `ranks`           | `Optional[List[int]]` | `None`  | The heading ranks, 1 to 6, that get ids; every heading when left out.                                    |
| `autolink`        | `Optional[Autolink]`  |         | Link each heading with an `id` to itself, as rehype-autolink-headings does: `true`, or its options.      |

**`Autolink`**

`html.slug`'s `autolink`: on, or on with rehype-autolink-headings' options.

Takes `Union[bool, AutolinkOptions]`.

**`AutolinkOptions`**

rehype-autolink-headings' options, JSON values only.

| Option              | Type                       | Default | Description                                                                                                                                                |
| ------------------- | -------------------------- | ------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `behavior`          | `AutolinkBehavior`         |         | Where the link goes: in the heading first or last, around its content, or before or after it.                                                              |
| `properties`        | `Optional[Dict[str, Any]]` | `None`  | The link's properties, as hast names them; `ariaHidden` and `tabIndex: -1` for `prepend` and `append` when left out.                                       |
| `content`           | `Any`                      | `None`  | hast content of the link, one node or a list; `<span class="icon icon-link">` when left out, except for `wrap`, which appends it to the heading's content. |
| `group`             | `Any`                      | `None`  | A hast element wrapping the heading and its link, for `before` and `after`.                                                                                |
| `headingProperties` | `Optional[Dict[str, Any]]` | `None`  | Properties set on each linked heading.                                                                                                                     |
| `test`              | `Optional[TagNames]`       |         | The tag names of the headings to link; every heading when left out.                                                                                        |

**`AutolinkBehavior`**

Where `autolink` puts each heading's link.

Takes `Literal["prepend", "append", "wrap", "before", "after"]`.

**`TagNames`**

One tag name or several.

Takes `Union[str, List[str]]`.

<!-- /generated -->

## html.externalLinks

Marks links to other sites as rehype-external-links does: `rel` (`nofollow` by default), `target`, more `properties`, and `content` appended in a `<span>`. A link counts as external when its URL is absolute in one of `protocols` (`http` and `https` by default) or protocol-relative.

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.to_html())
    .use(html.external_links({"target": "_blank", "rel": ["noopener"]}))
    .use(html.stringify())
    .process("See [npm](https://www.npmjs.com) or the [guide](/guide).")
)
file["value"]
# <p>See <a href="https://www.npmjs.com" rel="noopener" target="_blank">npm</a> or the <a href="/guide">guide</a>.</p>
```

<!-- generated:options html.externalLinks -->

Follows [rehype-external-links](https://www.npmjs.com/package/rehype-external-links).

| Option              | Type                       | Default | Description                                                                                                                             |
| ------------------- | -------------------------- | ------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `target`            | `Optional[str]`            | `None`  | The `target` of each external link, such as `_blank`.                                                                                   |
| `rel`               | `Optional[Tokens]`         |         | The `rel` of each external link: a list, or tokens separated by white space; `nofollow` when left out or empty, none for an empty list. |
| `protocols`         | `Optional[List[str]]`      | `None`  | The protocols of absolute URLs that count as external; `http` and `https` when left out. A protocol-relative `//…` URL always counts.   |
| `properties`        | `Optional[Dict[str, Any]]` | `None`  | More properties for each external link, as hast names them.                                                                             |
| `content`           | `Any`                      | `None`  | hast content appended to each external link in a `<span>`: one node or a list.                                                          |
| `contentProperties` | `Optional[Dict[str, Any]]` | `None`  | The properties of that `<span>`.                                                                                                        |

**`Tokens`**

Tokens: a list, or one string of them separated by white space.

Takes `Union[str, List[str]]`.

<!-- /generated -->

## html.unwrapImages

Replaces each paragraph that holds only images, links around them and white space with its content, as rehype-unwrap-images does, so images are not wrapped in `<p>`.

```python
from textlane import html, textlane

file = (
    textlane()
    .use(html.parse({"fragment": True}))
    .use(html.unwrap_images())
    .use(html.stringify())
    .process('<p><img src="a.png"></p>')
)
file["value"]
# <img src="a.png">
```

<!-- generated:options html.unwrapImages -->

Follows [rehype-unwrap-images](https://www.npmjs.com/package/rehype-unwrap-images).

No options.

<!-- /generated -->

## html.math

Turns `$…$` and `$$…$$` in text into the `<code class="language-math math-inline">` and `math-display` elements `md.math` makes, for `html.katex` to render. It reads dollars by pandoc's rule, so prices stay text: an opening `$` is followed by a non-space, a closing one follows a non-space and is not followed by a digit; `\$` is a dollar sign. Links, code, scripts, form text, `math` and `svg` are left alone, or the elements `ignore` names.

```python
from textlane import html, textlane

file = (
    textlane()
    .use(html.parse({"fragment": True}))
    .use(html.math())
    .use(html.stringify())
    .process(r'<p>Area $\pi r^2$, price $5</p>')
)
file["value"]
# <p>Area <code class="language-math math-inline">\pi r^2</code>, price $5</p>
```

<!-- generated:options html.math -->

| Option                 | Type                  | Default | Description                                                                                                       |
| ---------------------- | --------------------- | ------- | ----------------------------------------------------------------------------------------------------------------- |
| `singleDollarTextMath` | `bool`                | `True`  | Let one dollar, `$a$`, form math, as well as two.                                                                 |
| `ignore`               | `Optional[List[str]]` | `None`  | Elements, by tag name, whose text is left alone; links, code, scripts, form text, `math` and `svg` when left out. |
| `ignoreAttributes`     | `List[str]`           | `[]`    | Attributes, such as `data-literal`, whose elements' text is left alone.                                           |

<!-- /generated -->

## html.linkify

Turns bare URLs and email addresses in text into links, as GitHub's literal autolinks do: a URL with `://` as written, a `www.` one under `https://`, an email under `mailto:`. A bare domain, such as `index.html`, stays text. Links, code, scripts, form text, `math` and `svg` are left alone, or the elements `ignore` names, and any element with an attribute `ignoreAttributes` names.

```python
from textlane import html, textlane

file = (
    textlane()
    .use(html.parse({"fragment": True}))
    .use(html.linkify())
    .use(html.stringify())
    .process('<p>See www.example.com or a@example.com</p>')
)
file["value"]
# <p>See <a href="https://www.example.com">www.example.com</a> or <a href="mailto:a@example.com">a@example.com</a></p>
```

<!-- generated:options html.linkify -->

| Option             | Type                  | Default | Description                                                                                                       |
| ------------------ | --------------------- | ------- | ----------------------------------------------------------------------------------------------------------------- |
| `ignore`           | `Optional[List[str]]` | `None`  | Elements, by tag name, whose text is left alone; links, code, scripts, form text, `math` and `svg` when left out. |
| `ignoreAttributes` | `List[str]`           | `[]`    | Attributes, such as `data-literal`, whose elements' text is left alone.                                           |

<!-- /generated -->

## html.gemoji

Turns gemoji shortcodes in text, `:rocket:`, into their emoji, as `md.gemoji` does in Markdown; with `accessible`, each sits in a `<span role="img">` labelled with its name. Text that only looks like one, `10:30:00`, stays as written.

```python
from textlane import html, textlane

file = (
    textlane()
    .use(html.parse({"fragment": True}))
    .use(html.gemoji())
    .use(html.stringify())
    .process('<p>Ship it :rocket:</p>')
)
file["value"]
# <p>Ship it 🚀</p>
```

<!-- generated:options html.gemoji -->

| Option             | Type                  | Default | Description                                                                                                       |
| ------------------ | --------------------- | ------- | ----------------------------------------------------------------------------------------------------------------- |
| `accessible`       | `bool`                | `False` | Wrap each emoji in `<span role="img" aria-label="…">`, labelled with its name.                                    |
| `ignore`           | `Optional[List[str]]` | `None`  | Elements, by tag name, whose text is left alone; links, code, scripts, form text, `math` and `svg` when left out. |
| `ignoreAttributes` | `List[str]`           | `[]`    | Attributes, such as `data-literal`, whose elements' text is left alone.                                           |

<!-- /generated -->

## html.pathRewrite

Rewrites the URLs in `href`, `src`, `srcset` and `poster`, or the attributes `attributes` names, as a site serves them, as `md.pathRewrite` does for Markdown: a relative URL is resolved against the file's path and made root-relative, every root-relative one goes under `base` once, and an `href` naming a file in `pages` takes that page's URL.

```python
from textlane import html, textlane

file = (
    textlane()
    .use(html.parse({"fragment": True}))
    .use(html.path_rewrite({"base": "/docs"}))
    .use(html.stringify())
    .process('<a href="/guide.html">Guide</a>')
)
file["value"]
# <a href="/docs/guide.html">Guide</a>
```

<!-- generated:options html.pathRewrite -->

| Option       | Type                  | Default | Description                                                                                                              |
| ------------ | --------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------ |
| `root`       | `Optional[str]`       | `None`  | The directory URLs count from, as the file's path spells it; the path's own starting point when left out.                |
| `base`       | `Optional[str]`       | `None`  | The URL the site is served at, such as `/docs`, put before every root-relative URL once.                                 |
| `pages`      | `Dict[str, str]`      | `{}`    | Files, relative to `root`, to the URL of their page, such as `{"guide.html": "/guide/"}`; an `href` naming one takes it. |
| `attributes` | `Optional[List[str]]` | `None`  | The attributes whose URLs are rewritten; `href`, `src`, `srcset` and `poster` when left out.                             |

<!-- /generated -->

## html.extractToc

Stores the headings, nested by rank, as the file's `data.toc`, as @stefanprobst/rehype-extract-toc does: each `{ depth, value, id?, children? }`. Run it after `html.slug` so headings carry their ids.

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.to_html())
    .use(html.slug())
    .use(html.extract_toc())
    .use(html.stringify())
    .process("# Guide\n\n## Install")
)
file["data"]["toc"]
# [{ depth: 1, value: "Guide", id: "guide", children: [{ depth: 2, value: "Install", id: "install" }] }]
```

`"skip"` lists attributes that keep a heading out, such as `data-toc-hidden`; what would nest under it nests under the heading before. A `hidden` heading stays listed, which suits a heading meant for the table of contents alone.

<!-- generated:options html.extractToc -->

Follows [@stefanprobst/rehype-extract-toc](https://www.npmjs.com/package/@stefanprobst/rehype-extract-toc).

| Option       | Type            | Default | Description                                                                                                                                                                                                       |
| ------------ | --------------- | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `skip`       | `List[str]`     | `[]`    | Attributes, such as `data-toc-hidden`, that keep a heading out of the table of contents, or `name=value`, such as `data-toc=false`, for one value of it; what would nest under it nests under the heading before. |
| `skipInside` | `List[str]`     | `[]`    | Elements, by tag name, whose headings stay out, such as a card's title.                                                                                                                                           |
| `minDepth`   | `Optional[int]` | `None`  | The shallowest heading rank listed, 1 to 6.                                                                                                                                                                       |
| `maxDepth`   | `Optional[int]` | `None`  | The deepest heading rank listed, 1 to 6.                                                                                                                                                                          |

<!-- /generated -->

## html.highlight

Colors code blocks with TextMate grammars and themes, as Shiki does, natively.

````python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.to_html())
    .use(html.highlight({"theme": "github-light"}))
    .use(html.stringify())
    .process("```js\nlet a = 1\n```")
)
file["value"]
# <pre class="shiki github-light" style="background-color:#fff;color:#24292e"  …
````

`transformers` runs @shikijs/transformers natively, by name or with their options:
`notationDiff`, `notationFocus`, `notationHighlight` and `notationErrorLevel` read
`// [!code ++]`-style comments, `metaHighlight` reads `{1,3-5}` after the fence's language, `metaWordHighlight` marks every `/word/` it names
and `renderLineNumber` numbers lines, each marking lines with Shiki's classes. `"inline":
"tailing-curly-colon"` colors inline code written `` `a(){:js}` ``, `colorReplacements` and
`addLanguageClass` are Shiki's own, and `metaProperty` puts a fence's meta on its `<pre>`.

```python
html.highlight({
    "themes": {"light": "github-light", "dark": "github-dark"},
    "transformers": ["notationDiff", {"name": "renderLineNumber", "start": 10}],
})
```

A grammar's time grows with the square of a line's length, so a line of 2,000 characters or more, typically minified code, is left one plain token, as Shiki's `tokenizeMaxLineLength` leaves it; Shiki has no such limit by default, and `0` lifts it.

<!-- generated:options html.highlight -->

Follows [@shikijs/rehype](https://www.npmjs.com/package/@shikijs/rehype).

| Option                  | Type                          | Default | Description                                                                                                                                                                                                                          |
| ----------------------- | ----------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `themes`                | `Optional[Dict[str, Any]]`    | `None`  | The theme of each color mode, `light` first, as `{"light": …, "dark": …}`.                                                                                                                                                           |
| `theme`                 | `Optional[str]`               | `None`  | The one theme, when `themes` is not given.                                                                                                                                                                                           |
| `defaultLanguage`       | `Optional[str]`               | `None`  | The language of a block that names none; such a block is left alone without it.                                                                                                                                                      |
| `fallbackLanguage`      | `Optional[str]`               | `None`  | The language of a block naming one Shiki does not know; left alone without it.                                                                                                                                                       |
| `colorReplacements`     | `Dict[str, ColorReplacement]` |         | Colors to replace, by their lowercase spelling: a color for every theme, or a theme's name to a map of its own.                                                                                                                      |
| `addLanguageClass`      | `bool`                        | `False` | Add `language-<name>` to each highlighted `<code>`.                                                                                                                                                                                  |
| `inline`                | `Optional[InlineCode]`        |         | Highlight inline code too, by how it names its language: `tailing-curly-colon` reads `` `a(){:js}` ``.                                                                                                                               |
| `metaProperty`          | `Optional[str]`               | `None`  | The `<pre>` property that gets a block's meta, the text after its language; the JSON stand-in for `parseMetaString: meta => ({ [name]: meta })`.                                                                                     |
| `transformers`          | `List[HighlightTransformer]`  |         | The transformers of @shikijs/transformers to run, in order, by name or with options.                                                                                                                                                 |
| `tokenizeMaxLineLength` | `Optional[int]`               | `None`  | Lines at least this long, in UTF-16 units, are left one plain token as Shiki's own option leaves them; 2,000 when left out, where Shiki has no limit, since a grammar's time grows with the square of a line's length. `0` lifts it. |
| `lineAttributes`        | `Dict[str, LineMark]`         |         | Attributes of a block's `<code>` or `<pre>` listing lines, `2,4-5`, each to the classes those lines get, or to `{line, pre}`, the second for the `<pre>`: `{"data-focus": {"line": "focused", "pre": "has-focused"}}`.               |
| `wordAttribute`         | `Optional[str]`               | `None`  | The attribute of a block's `<code>` or `<pre>` listing words, separated by commas, marked at every place as `metaWordHighlight` marks a meta's `/word/`.                                                                             |
| `lineNumberAttribute`   | `Optional[str]`               | `None`  | The attribute of a block's `<code>` or `<pre>` that numbers its lines as `renderLineNumber` does, from its value, or 1 when it is empty.                                                                                             |
| `skipElements`          | `bool`                        | `False` | Leave a block alone when its code holds elements, such as `<mark>`, which highlighting would flatten into text.                                                                                                                      |
| `keepProperties`        | `bool`                        | `False` | Keep the properties of a block's `<pre>` and `<code>`, such as `data-title`, on the ones Shiki makes, merging classes; Shiki's own win where both set one.                                                                           |

**`ColorReplacement`**

A replacement color, or a theme's own replacements.

Takes `Union[str, Dict[str, str]]`.

**`InlineCode`**

How inline code names its language.

Takes `Union[Literal["tailing-curly-colon"], Literal["class"]]`.

**`HighlightTransformer`**

A transformer of @shikijs/transformers: its name alone, or its name and options.

Takes `Union[TransformerName, ConfiguredTransformer]`.

**`LineMark`**

The classes the lines a `lineAttributes` attribute lists get, and maybe its `<pre>`.

Takes `Union[Classes, Inline1]`.

**`TransformerName`**

The transformers `html.highlight` runs natively.

Takes `Literal["notationDiff", "notationFocus", "notationHighlight", "notationErrorLevel", "metaHighlight", "metaWordHighlight", "renderLineNumber"]`.

**`ConfiguredTransformer`**

A transformer with its options, as @shikijs/transformers names them.

Takes `Union[Inline0, Inline1, Inline2, Inline3, Inline4, Inline5, Inline6]`.

**`Classes`**

Class names: a list, or one string of them separated by white space.

Takes `Union[str, List[str]]`.

<!-- /generated -->

## html.katex

Renders math with KaTeX, as rehype-katex does, natively: each `$…$`, `$$…$$` and ` ```math ` that `md.math` and `md.toHtml` leave becomes KaTeX's HTML and MathML. Pages need KaTeX's stylesheet and fonts. A formula that fails to render is reported in the file's messages and shown in `errorColor`.

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.math())
    .use(md.to_html())
    .use(html.katex())
    .use(html.stringify())
    .process("Euler: $e^{i\\pi} = -1$")
)
file["value"]
# <p>Euler: <span class="katex"><span class="katex-mathml"><math …
```

<!-- generated:options html.katex -->

Follows [rehype-katex](https://www.npmjs.com/package/rehype-katex).

| Option             | Type                    | Default | Description                                                                                                                                                   |
| ------------------ | ----------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `output`           | `Optional[KatexOutput]` |         | The markup to write, both HTML and MathML by default.                                                                                                         |
| `leqno`            | `bool`                  | `False` | Put equation numbers on the left.                                                                                                                             |
| `fleqn`            | `bool`                  | `False` | Align display math to the left instead of centering it.                                                                                                       |
| `errorColor`       | `Optional[str]`         | `None`  | The color of math that fails to render, `#cc0000` by default.                                                                                                 |
| `macros`           | `Dict[str, str]`        | `{}`    | Macros by name, each to the TeX it expands to, as `{"\\RR": "\\mathbb{R}"}`.                                                                                  |
| `minRuleThickness` | `Optional[float]`       | `None`  | The thinnest a rule, such as a fraction line, is drawn, in ems.                                                                                               |
| `colorIsTextColor` | `bool`                  | `False` | Let `\color` color the text after it, as LaTeX does.                                                                                                          |
| `strict`           | `Optional[KatexStrict]` |         | What to do with input LaTeX would reject: `true` or `"error"` fails it, `"warn"` prints a warning, `false` or `"ignore"` (the default) accepts it.            |
| `trust`            | `bool`                  | `False` | Allow commands that can run untrusted input, such as `\href` and `\includegraphics`.                                                                          |
| `maxSize`          | `Optional[float]`       | `None`  | The largest a user-given size may be, in ems.                                                                                                                 |
| `maxExpand`        | `Optional[int]`         | `None`  | How many macro expansions one formula may take, 1000 by default.                                                                                              |
| `globalGroup`      | `bool`                  | `False` | Let definitions such as `\def` last past the formula they are in.                                                                                             |
| `tagNames`         | `List[str]`             | `[]`    | Elements, by tag name, that hold math besides the classed ones, such as `my-math`; one is display math when it has a `display` attribute other than `inline`. |

**`KatexOutput`**

The markup KaTeX writes.

Takes `Literal["html", "mathml", "htmlAndMathml"]`.

**`KatexStrict`**

How strictly KaTeX holds input to LaTeX.

Takes `Union[bool, KatexStrictMode]`.

**`KatexStrictMode`**

One level of KaTeX's strictness.

Takes `Literal["ignore", "warn", "error"]`.

<!-- /generated -->

## html.toMarkdown

Turns hast into mdast, so HTML can be written out as Markdown.

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(html.parse({"fragment": True}))
    .use(html.to_markdown())
    .use(md.stringify())
    .process("<h2>Title</h2><p>Some <strong>bold</strong> text.</p>")
)
file["value"]
# ## Title
#
# Some **bold** text.
```

<!-- generated:options html.toMarkdown -->

Follows [rehype-remark](https://www.npmjs.com/package/rehype-remark).

| Option      | Type                  | Default | Description                                                                                                                                                                                                 |
| ----------- | --------------------- | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `document`  | `bool`                | `False` | Wrap all phrasing content of the root in paragraphs, as for a whole document.                                                                                                                               |
| `newlines`  | `bool`                | `False` | Collapse whitespace holding a line ending to that line ending instead of a space.                                                                                                                           |
| `checked`   | `Optional[str]`       | `None`  | The text of a checked checkbox or radio, `[x]` by default.                                                                                                                                                  |
| `unchecked` | `Optional[str]`       | `None`  | The text of an unchecked checkbox or radio, `[ ]` by default.                                                                                                                                               |
| `quotes`    | `Optional[List[str]]` | `None`  | The quotes `<q>` uses, an opening and closing character each, alternating by how deep it nests; `"` by default.                                                                                             |
| `math`      | `bool`                | `False` | Turn MathML `<math>` holding a TeX annotation, as `html.katex` writes it, into math nodes, `$…$` once `md.math` writes them, display math for `display="block"`, and drop KaTeX's visual `katex-html` copy. |
| `codeMeta`  | `Dict[str, str]`      | `{}`    | Attributes of a code block's `<pre>` or `<code>` written into its meta, each to the key it takes there: `{"data-title": "title"}` makes `title="a.js"`.                                                     |

<!-- /generated -->

## html.toNaturalLanguage

Turns hast into nlcst, so `nl.*` plugins can check the prose of an HTML document.

```python
from textlane import html, nl, textlane

file = (
    textlane()
    .use(html.parse({"fragment": True}))
    .use(html.to_natural_language())
    .use(nl.repeated_words())
    .use(nl.stringify())
    .process("<p>It is the the best.</p>")
)
file["messages"][0]["reason"]
# Unexpected repeated `the`, remove one occurrence
```

<!-- generated:options html.toNaturalLanguage -->

Follows [rehype-retext](https://www.npmjs.com/package/rehype-retext).

No options.

<!-- /generated -->

## html.stringify

Serializes hast into HTML; its options pick quoting, closing and escaping.

```python
from textlane import html, textlane

file = (
    textlane()
    .use(html.parse({"fragment": True}))
    .use(html.stringify({"quote": "'"}))
    .process("<img alt=\"Logo\" src=\"logo.svg\">")
)
file["value"]
# <img alt='Logo' src='logo.svg'>
```

<!-- generated:options html.stringify -->

Follows [rehype-stringify](https://www.npmjs.com/package/rehype-stringify).

| Option                     | Type                  | Default | Description                                                                              |
| -------------------------- | --------------------- | ------- | ---------------------------------------------------------------------------------------- |
| `allowDangerousHtml`       | `bool`                | `False` | Write raw nodes as they are instead of escaping them.                                    |
| `allowDangerousCharacters` | `bool`                | `False` | Leave characters that are only dangerous in some contexts, such as `` ` ``, as they are. |
| `allowParseErrors`         | `bool`                | `False` | Leave characters as they are where browsers accept them though the spec does not.        |
| `bogusComments`            | `bool`                | `False` | Write comments as bogus comments, `<?a>`.                                                |
| `closeEmptyElements`       | `bool`                | `False` | Close empty foreign elements, `<a />`.                                                   |
| `closeSelfClosing`         | `bool`                | `False` | Close void elements, `<br />`.                                                           |
| `collapseEmptyAttributes`  | `bool`                | `False` | Write empty attributes as bare names, `a` rather than `a=""`.                            |
| `omitOptionalTags`         | `bool`                | `False` | Leave out tags the HTML spec lets a parser imply.                                        |
| `preferUnquoted`           | `bool`                | `False` | Leave attribute values unquoted where that is safe.                                      |
| `quote`                    | `Optional[str]`       | `None`  | The quote attribute values are wrapped in: `"` or `'`.                                   |
| `quoteSmart`               | `bool`                | `False` | Use the other quote when the value holds more of the preferred one.                      |
| `space`                    | `Optional[str]`       | `None`  | The space the tree is in: `html` or `svg`.                                               |
| `tightAttributes`          | `bool`                | `False` | Leave out the space between quoted attributes.                                           |
| `tightCommaSeparatedLists` | `bool`                | `False` | Join comma-separated values with `,` rather than `, `.                                   |
| `tightDoctype`             | `bool`                | `False` | Write `<!doctypehtml>`.                                                                  |
| `tightSelfClosing`         | `bool`                | `False` | Leave out the space before `/>` when the attributes allow.                               |
| `upperDoctype`             | `bool`                | `False` | Write `<!DOCTYPE html>`.                                                                 |
| `voids`                    | `Optional[List[str]]` | `None`  | The tag names of void elements, which have no closing tag.                               |

<!-- /generated -->
