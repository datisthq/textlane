---
title: Markdown plugins
description: Every Markdown plugin, its options and the unified package it follows.
path: /python/markdown-plugins/
icon: file-text
order: 6
---

# Markdown plugins

Markdown plugins parse Markdown into mdast, extend its syntax and turn mdast into Markdown, HTML trees or natural-language trees. `md.parse` and `md.stringify` are the parser and the compiler; a syntax extension such as `md.gfm` teaches both, so add it beside either.

## md.parse

Parses Markdown into an mdast tree, CommonMark as micromark reads it; add syntax extensions such as `md.gfm` beside it.

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.to_html())
    .use(html.stringify())
    .process_sync("# Hello *world*")
)
file["value"]
# <h1>Hello <em>world</em></h1>
```

<!-- generated:options md.parse -->

Follows [remark-parse](https://www.npmjs.com/package/remark-parse).

No options.

<!-- /generated -->

## md.directive

Adds generic directives — `:text`, `::leaf` and `:::container` — read by `md.parse` and written by `md.stringify`.

```python
from textlane import md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.directive())
    .use(md.stringify())
    .process_sync(":::note{.info}\nRead *this*.\n:::")
)
file["value"]
# :::note{.info}
# Read *this*.
# :::
```

<!-- generated:options md.directive -->

Follows [remark-directive](https://www.npmjs.com/package/remark-directive).

| Option                    | Type            | Default | Description                                                                     |
| ------------------------- | --------------- | ------- | ------------------------------------------------------------------------------- |
| `collapseEmptyAttributes` | `bool`          | `True`  | Write an empty attribute as its name alone, `{a}`, rather than `{a=""}`.        |
| `preferShortcut`          | `bool`          | `True`  | Write `id` and `class` as `#a` and `.b` where they can be.                      |
| `preferUnquoted`          | `bool`          | `False` | Leave attribute values unquoted where they can be.                              |
| `quote`                   | `Optional[str]` | `None`  | The quote of attribute values: `"` or `'`; `md.stringify`'s `quote` when unset. |
| `quoteSmart`              | `bool`          | `False` | Use the other quote when the preferred one occurs more often in a value.        |

<!-- /generated -->

## md.gfm

Adds GitHub Flavored Markdown: tables, task lists, strikethrough, autolink literals and footnotes.

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.gfm())
    .use(md.to_html())
    .use(html.stringify())
    .process_sync("~~old~~ www.example.com")
)
file["value"]
# <p><del>old</del> <a href="http://www.example.com">www.example.com</a></p>
```

<!-- generated:options md.gfm -->

Follows [remark-gfm](https://www.npmjs.com/package/remark-gfm).

| Option             | Type   | Default | Description                                                                |
| ------------------ | ------ | ------- | -------------------------------------------------------------------------- |
| `singleTilde`      | `bool` | `True`  | Let one tilde, `~a~`, form strikethrough, as well as two.                  |
| `tableCellPadding` | `bool` | `True`  | Pad table cells with a space on each side when writing Markdown.           |
| `tablePipeAlign`   | `bool` | `True`  | Line up table pipes when writing Markdown.                                 |
| `firstLineBlank`   | `bool` | `False` | Write a footnote definition's first line blank when it holds flow content. |

<!-- /generated -->

## md.definitionList

Adds definition lists: a term, then lines starting with `: ` that define it.

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.definition_list())
    .use(md.to_html())
    .use(html.stringify())
    .process_sync("Apple\n: A fruit.")
)
file["value"]
# <dl>
# <dt>Apple</dt>
# <dd>A fruit.
# </dd>
# </dl>
```

<!-- generated:options md.definitionList -->

Follows [remark-definition-list](https://www.npmjs.com/package/remark-definition-list).

No options.

<!-- /generated -->

## md.math

Adds math: `$$` blocks and `$` inline, kept as math nodes for a later stage to render.

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.math())
    .use(md.to_html())
    .use(html.stringify())
    .process_sync("Euler: $e^{i\\pi} = -1$")
)
file["value"]
# <p>Euler: <code class="language-math math-inline">e^{i\pi} = -1</code></p>
```

<!-- generated:options md.math -->

Follows [remark-math](https://www.npmjs.com/package/remark-math).

| Option                 | Type   | Default | Description                                               |
| ---------------------- | ------ | ------- | --------------------------------------------------------- |
| `singleDollarTextMath` | `bool` | `True`  | Let one dollar, `$a$`, form math in text, as well as two. |

<!-- /generated -->

## md.mdx

Adds MDX syntax — JSX, `{expressions}` and `import`/`export` — and turns off indented code, autolinks and raw HTML, as MDX does.

```python
from textlane import md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.mdx())
    .use(md.stringify())
    .process_sync("import {Chart} from './chart.js'\n\n# Sales {year}\n\n<Chart  data={rows}/>")
)
file["value"]
# import {Chart} from './chart.js'
#
# # Sales {year}
#
# <Chart data={rows} />
```

<!-- generated:options md.mdx -->

Follows [remark-mdx](https://www.npmjs.com/package/remark-mdx).

| Option             | Type              | Default | Description                                                                |
| ------------------ | ----------------- | ------- | -------------------------------------------------------------------------- |
| `quote`            | `Optional[str]`   | `None`  | The quote of JSX attribute values: `"` (when unset) or `'`.                |
| `quoteSmart`       | `bool`            | `False` | Use the other quote when the preferred one occurs more often in a value.   |
| `tightSelfClosing` | `bool`            | `False` | Write self-closing tags without a space before the slash, `<a/>`.          |
| `printWidth`       | `Optional[float]` | `None`  | The width past which a flow element's attributes go on lines of their own. |

<!-- /generated -->

## md.frontmatter

Reads frontmatter — YAML by default, TOML or your own fences by option — into a node of its own.

```python
from textlane import md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.frontmatter(["yaml", "toml"]))
    .use(md.stringify())
    .process_sync("+++\ntitle = \"Hi\"\n+++\n\n# Hi")
)
file["value"]
# +++
# title = "Hi"
# +++
#
# # Hi
```

<!-- generated:options md.frontmatter -->

Follows [remark-frontmatter](https://www.npmjs.com/package/remark-frontmatter).

The kinds of matter to recognize, as remark-frontmatter takes them: a preset, a matter, or a list of them; `yaml` when unset.

Takes `Union[Literal["yaml", "toml"], Matter, List[Union[Literal["yaml", "toml"], Matter]]]`.

**`Matter`**

One kind of frontmatter: the node `type` it becomes and its fences, a `marker` repeated three times or a whole `fence`.

| Option     | Type    | Default | Description |
| ---------- | ------- | ------- | ----------- |
| `type`     | `str`   |         |             |
| `marker`   | `Fence` |         |             |
| `fence`    | `Fence` |         |             |
| `anywhere` | `bool`  |         |             |

**`Fence`**

A fence, or its opening and closing ones.

Takes `Union[str, FencePair]`.

**`FencePair`**

Different opening and closing fences.

| Option  | Type  | Default | Description |
| ------- | ----- | ------- | ----------- |
| `open`  | `str` |         |             |
| `close` | `str` |         |             |

<!-- /generated -->

## md.gemoji

Turns gemoji shortcodes such as `:tada:` into the emoji they name.

```python
from textlane import md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.gemoji())
    .use(md.stringify())
    .process_sync("Shipped :tada:")
)
file["value"]
# Shipped 🎉
```

<!-- generated:options md.gemoji -->

Follows [remark-gemoji](https://www.npmjs.com/package/remark-gemoji).

No options.

<!-- /generated -->

## md.stringify

Serializes mdast back into Markdown; its options pick the markers and spacing it writes.

```python
from textlane import md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.stringify({"bullet": "-", "emphasis": "_"}))
    .process_sync("* *one*\n* two")
)
file["value"]
# - _one_
# - two
```

<!-- generated:options md.stringify -->

Follows [remark-stringify](https://www.npmjs.com/package/remark-stringify).

| Option                | Type                  | Default | Description                                                 |
| --------------------- | --------------------- | ------- | ----------------------------------------------------------- |
| `bullet`              | `Optional[str]`       | `None`  | The marker of unordered list items: `*`, `+` or `-`.        |
| `bulletOrdered`       | `Optional[str]`       | `None`  | The marker of ordered list items: `.` or `)`.               |
| `bulletOther`         | `Optional[str]`       | `None`  | The marker of a list right after a list with `bullet`.      |
| `closeAtx`            | `bool`                | `False` | Close ATX headings with as many `#` as they open with.      |
| `emphasis`            | `Optional[str]`       | `None`  | The marker of emphasis: `*` or `_`.                         |
| `fence`               | `Optional[str]`       | `None`  | The marker of fenced code: `` ` `` or `~`.                  |
| `fences`              | `bool`                | `True`  | Fence code; `false` indents it where it can.                |
| `incrementListMarker` | `bool`                | `True`  | Number ordered list items up from `start`.                  |
| `listItemIndent`      | `Optional[str]`       | `None`  | How list item content is indented: `one`, `tab` or `mixed`. |
| `quote`               | `Optional[str]`       | `None`  | The marker of titles: `"` or `'`.                           |
| `resourceLink`        | `bool`                | `False` | Write links as resources even where an autolink would do.   |
| `rule`                | `Optional[str]`       | `None`  | The marker of thematic breaks: `*`, `-` or `_`.             |
| `ruleRepetition`      | `Optional[int]`       | `None`  | How many markers a thematic break has, 3 or more.           |
| `ruleSpaces`          | `bool`                | `False` | Put spaces between the markers of a thematic break.         |
| `setext`              | `bool`                | `False` | Write headings of rank 1 and 2 as setext headings.          |
| `strong`              | `Optional[str]`       | `None`  | The marker of strong: `*` or `_`.                           |
| `tightDefinitions`    | `bool`                | `False` | Join definitions with no blank line between them.           |
| `unsafe`              | `List[UnsafePattern]` |         | Extra patterns of characters to escape.                     |

**`UnsafePattern`**

A character to escape where it would otherwise start or end syntax.

| Option           | Type              | Default | Description                                                      |
| ---------------- | ----------------- | ------- | ---------------------------------------------------------------- |
| `character`      | `str`             | `""`    | The character, escaped as a regular expression would need it.    |
| `before`         | `Optional[str]`   | `None`  | A regular expression that must match right before the character. |
| `after`          | `Optional[str]`   | `None`  | A regular expression that must match right after the character.  |
| `atBreak`        | `bool`            | `False` | Only at the start of a line, after any indent.                   |
| `inConstruct`    | `Optional[Names]` |         | Only inside one of these constructs.                             |
| `notInConstruct` | `Optional[Names]` |         | Never inside one of these constructs.                            |
| `beforeNode`     | `Optional[Names]` |         | Only at the start of a node right after one of these node types. |
| `afterNode`      | `Optional[Names]` |         | Only at the end of a node right before one of these node types.  |
| `unicode`        | `bool`            | `False` | Read `before` and `after` with Unicode classes such as `\p{L}`.  |

**`Names`**

One name or several.

Takes `Union[str, List[str]]`.

<!-- /generated -->

## md.toHtml

Turns mdast into hast, the HTML tree, so HTML plugins and `html.stringify` can follow.

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.to_html({"allowDangerousHtml": True}))
    .use(html.stringify({"allowDangerousHtml": True}))
    .process_sync("Hi <kbd>Ctrl</kbd>")
)
file["value"]
# <p>Hi <kbd>Ctrl</kbd></p>
```

<!-- generated:options md.toHtml -->

Follows [remark-rehype](https://www.npmjs.com/package/remark-rehype).

| Option                 | Type            | Default | Description                                                                                     |
| ---------------------- | --------------- | ------- | ----------------------------------------------------------------------------------------------- |
| `allowDangerousHtml`   | `bool`          | `False` | Keep raw HTML in the tree instead of dropping it.                                               |
| `clobberPrefix`        | `Optional[str]` | `None`  | What footnote `id`s start with, so they cannot clobber the page's; `user-content-` when unset.  |
| `footnoteBackContent`  | `Optional[str]` | `None`  | The text of each footnote's back reference; `↩`, numbered from the second, when unset.          |
| `footnoteBackLabel`    | `Optional[str]` | `None`  | The `aria-label` of each footnote's back reference; `Back to reference 1` and so on when unset. |
| `footnoteLabel`        | `Optional[str]` | `None`  | The heading of the footnotes section; `Footnotes` when unset.                                   |
| `footnoteLabelTagName` | `Optional[str]` | `None`  | The tag name of the footnotes section's heading; `h2` when unset.                               |
| `passThrough`          | `List[str]`     | `[]`    | Node types without a handler to copy into hast as they are, their children converted.           |

<!-- /generated -->

## md.toNaturalLanguage

Turns mdast into nlcst, the natural-language tree, so `nl.*` plugins can check the prose of a Markdown document.

```python
from textlane import md, nl, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.to_natural_language())
    .use(nl.repeated_words())
    .use(nl.stringify())
    .process_sync("It is *the the* best.")
)
file["messages"][0]["reason"]
# Unexpected repeated `the`, remove one occurrence
```

<!-- generated:options md.toNaturalLanguage -->

Follows [remark-retext](https://www.npmjs.com/package/remark-retext).

| Option   | Type        | Default | Description                                                                        |
| -------- | ----------- | ------- | ---------------------------------------------------------------------------------- |
| `ignore` | `List[str]` | `[]`    | Node types to leave out, besides `table`, `tableRow` and `tableCell`.              |
| `source` | `List[str]` | `[]`    | Node types to keep as one `SourceNode` of their source text, besides `inlineCode`. |

<!-- /generated -->
