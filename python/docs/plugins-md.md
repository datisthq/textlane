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
    .process("# Hello *world*")
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
    .process(":::note{.info}\nRead *this*.\n:::")
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
    .process("~~old~~ www.example.com")
)
file["value"]
# <p><del>old</del> <a href="http://www.example.com">www.example.com</a></p>
```

`"alerts": True` also renders GitHub alerts, a block quote opening with `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]` or `[!CAUTION]`, as GitHub does: a `<div class="markdown-alert markdown-alert-note">` with a title and its Octicon, as remark-github-blockquote-alert writes it. For an MDX component instead, use `md.component`'s `alerts`.

<!-- generated:options md.gfm -->

Follows [remark-gfm](https://www.npmjs.com/package/remark-gfm).

| Option             | Type               | Default | Description                                                                                                                                 |
| ------------------ | ------------------ | ------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `singleTilde`      | `bool`             | `True`  | Let one tilde, `~a~`, form strikethrough, as well as two.                                                                                   |
| `tableCellPadding` | `bool`             | `True`  | Pad table cells with a space on each side when writing Markdown.                                                                            |
| `tablePipeAlign`   | `bool`             | `True`  | Line up table pipes when writing Markdown.                                                                                                  |
| `firstLineBlank`   | `bool`             | `False` | Write a footnote definition's first line blank when it holds flow content.                                                                  |
| `alerts`           | `Optional[Alerts]` |         | Render GitHub alerts, a block quote opening with `[!NOTE]`, as GitHub does, as remark-github-blockquote-alert does: `true`, or its options. |

**`Alerts`**

`md.gfm`'s `alerts`: on, or on with remark-github-blockquote-alert's options.

Takes `Union[bool, AlertsOptions]`.

**`AlertsOptions`**

remark-github-blockquote-alert's options.

| Option        | Type   | Default | Description                                                                    |
| ------------- | ------ | ------- | ------------------------------------------------------------------------------ |
| `legacyTitle` | `bool` | `False` | Read a title after the type, `[!NOTE/Read this]`, in place of the type's name. |
| `tagName`     | `str`  | `"div"` | The tag name of the alert's element.                                           |
| `classNames`  | `str`  | `""`    | More classes for the alert's element, separated by spaces.                     |

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
    .process("Apple\n: A fruit.")
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
    .process("Euler: $e^{i\\pi} = -1$")
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
    .process("import {Chart} from './chart.js'\n\n# Sales {year}\n\n<Chart  data={rows}/>")
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
    .process("+++\ntitle = \"Hi\"\n+++\n\n# Hi")
)
file["value"]
# +++
# title = "Hi"
# +++
#
# # Hi
```

With `data: true` it also parses the YAML a document starts with into the file's `data.matter`, as vfile-matter does (`{}` when there is none; YAML 1.2's core schema, as the `yaml` package reads it), and `strip: true` drops that node from the tree. Invalid YAML fails with `invalid-frontmatter`. To combine them with other matters, pass `{ matters, data, strip }`.

```python
with_data = (
    textlane()
    .use(md.parse())
    .use(md.frontmatter({"data": True}))
    .use(md.stringify())
    .process("---\ntitle: Hello\ntags: [a, b]\n---\n\n# Hello")
)
with_data["data"]["matter"]
# {'title': 'Hello', 'tags': ['a', 'b']}
```

<!-- generated:options md.frontmatter -->

Follows [remark-frontmatter](https://www.npmjs.com/package/remark-frontmatter).

The kinds of matter to recognize, as remark-frontmatter takes them (a preset, a matter, or a list of them; `yaml` when unset), or textlane's options object.

Takes `Union[Literal["yaml", "toml"], Matter, List[Union[Literal["yaml", "toml"], Matter]], FrontmatterSettings]`.

**`Matter`**

One kind of frontmatter: the node `type` it becomes and its fences, a `marker` repeated three times or a whole `fence`.

| Option     | Type    | Default | Description |
| ---------- | ------- | ------- | ----------- |
| `type`     | `str`   |         |             |
| `marker`   | `Fence` |         |             |
| `fence`    | `Fence` |         |             |
| `anywhere` | `bool`  |         |             |

**`FrontmatterSettings`**

textlane's options: the matters, and what to do with YAML frontmatter.

| Option    | Type                                                                                   | Default | Description                                                                                                               |
| --------- | -------------------------------------------------------------------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------- |
| `matters` | `Union[Literal["yaml", "toml"], Matter, List[Union[Literal["yaml", "toml"], Matter]]]` |         | The kinds of matter to recognize, as remark-frontmatter takes them; `yaml` when unset.                                    |
| `data`    | `bool`                                                                                 |         | Parse the YAML frontmatter a document starts with into `file.data.matter`, `{}` when there is none, as vfile-matter does. |
| `strip`   | `bool`                                                                                 |         | Remove the YAML frontmatter a document starts with from the tree.                                                         |

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

## md.cjkFriendly

Lets `*`, `_` and, with `md.gfm`, `~` emphasize text next to Chinese, Japanese and Korean characters and punctuation, as remark-cjk-friendly and remark-cjk-friendly-gfm-strikethrough parse it: `**これは強調。**です` is strong, where CommonMark leaves the asterisks. Text without CJK parses as before. `md.stringify` does not yet escape emphasis the CJK-friendly way, so Markdown written back may not parse to the same tree.

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.cjk_friendly())
    .use(md.to_html())
    .use(html.stringify())
    .process("**これは強調。**です")
)
file["value"]
# <p><strong>これは強調。</strong>です</p>
```

<!-- generated:options md.cjkFriendly -->

Follows [remark-cjk-friendly](https://www.npmjs.com/package/remark-cjk-friendly).

No options.

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
    .process("Shipped :tada:")
)
file["value"]
# Shipped 🎉
```

`"emoticon": True` also turns emoticons such as `:)` and `:-(` into emoji, `"accessible": True` wraps an emoji in `<span role="img" aria-label="…">` and `"padSpaceAfter": True` puts a space after each, as remark-emoji's options do. Shortcodes stay gemoji's names, where remark-emoji reads node-emoji's, which name a few emoji differently.

<!-- generated:options md.gemoji -->

Follows [remark-gemoji](https://www.npmjs.com/package/remark-gemoji).

| Option          | Type   | Default | Description                                                                    |
| --------------- | ------ | ------- | ------------------------------------------------------------------------------ |
| `emoticon`      | `bool` | `False` | Also turn emoticons such as `:)` and `:-(` into emoji, as remark-emoji does.   |
| `accessible`    | `bool` | `False` | Wrap each emoji in `<span role="img" aria-label="…">`, labelled with its name. |
| `padSpaceAfter` | `bool` | `False` | Put a space after each emoji.                                                  |

<!-- /generated -->

## md.smartypants

Turns straight quotes, dashes, dots and backticks in Markdown text into their typographic forms, pairing quotes across links and emphasis, as remark-smartypants does. Code is left alone.

```python
from textlane import md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.smartypants())
    .use(md.stringify())
    .process("\"Hello\" -- it's *done*...")
)
file["value"]
# “Hello” — it’s *done*…
```

<!-- generated:options md.smartypants -->

Follows [remark-smartypants](https://www.npmjs.com/package/remark-smartypants).

| Option          | Type                        | Default | Description                                                                                                                 |
| --------------- | --------------------------- | ------- | --------------------------------------------------------------------------------------------------------------------------- |
| `backticks`     | `Backticks`                 |         | Turn double backticks and double straight single quotes into double quotes; `all` turns single ones into single quotes too. |
| `closingQuotes` | `Optional[QuoteCharacters]` |         | The closing quotes to use: `”` and `’` by default.                                                                          |
| `dashes`        | `Dashes`                    |         | Turn two dashes into an em dash; `oldschool` and `inverted` treat three too.                                                |
| `ellipses`      | `Ellipses`                  |         | Turn three dots into an ellipsis, with spaces between them or without.                                                      |
| `openingQuotes` | `Optional[QuoteCharacters]` |         | The opening quotes to use: `“` and `‘` by default.                                                                          |
| `quotes`        | `Optional[bool]`            | `True`  | Turn straight quotes into smart ones.                                                                                       |

**`Backticks`**

Takes `Union[bool, Literal["all"]]`.

**`QuoteCharacters`**

The quote characters to use for double and single quotes.

| Option   | Type  | Default | Description |
| -------- | ----- | ------- | ----------- |
| `double` | `str` |         |             |
| `single` | `str` |         |             |

**`Dashes`**

Takes `Union[bool, Literal["oldschool", "inverted"]]`.

**`Ellipses`**

Takes `Union[bool, Literal["spaced", "unspaced"]]`.

<!-- /generated -->

## md.smartypants

Turns straight quotes, dashes, dots and backticks in Markdown text into their typographic forms, pairing quotes across links and emphasis, as remark-smartypants does. Code is left alone.

```python
from textlane import md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.smartypants())
    .use(md.stringify())
    .process("\"Hello\" -- it's *done*...")
)
file["value"]
# “Hello” — it’s *done*…
```

<!-- generated:options md.smartypants -->

Follows [remark-smartypants](https://www.npmjs.com/package/remark-smartypants).

| Option          | Type                        | Default | Description                                                                                                                 |
| --------------- | --------------------------- | ------- | --------------------------------------------------------------------------------------------------------------------------- |
| `backticks`     | `Backticks`                 |         | Turn double backticks and double straight single quotes into double quotes; `all` turns single ones into single quotes too. |
| `closingQuotes` | `Optional[QuoteCharacters]` |         | The closing quotes to use: `”` and `’` by default.                                                                          |
| `dashes`        | `Dashes`                    |         | Turn two dashes into an em dash; `oldschool` and `inverted` treat three too.                                                |
| `ellipses`      | `Ellipses`                  |         | Turn three dots into an ellipsis, with spaces between them or without.                                                      |
| `openingQuotes` | `Optional[QuoteCharacters]` |         | The opening quotes to use: `“` and `‘` by default.                                                                          |
| `quotes`        | `Optional[bool]`            | `True`  | Turn straight quotes into smart ones.                                                                                       |

**`Backticks`**

Takes `Union[bool, Literal["all"]]`.

**`QuoteCharacters`**

The quote characters to use for double and single quotes.

| Option   | Type  | Default | Description |
| -------- | ----- | ------- | ----------- |
| `double` | `str` |         |             |
| `single` | `str` |         |             |

**`Dashes`**

Takes `Union[bool, Literal["oldschool", "inverted"]]`.

**`Ellipses`**

Takes `Union[bool, Literal["spaced", "unspaced"]]`.

<!-- /generated -->

## md.component

Turns directives, GitHub alerts and other nodes into MDX JSX elements, the components an MDX
runtime renders, by declarative rules, so a site's `:::note` or `::button` needs no plugin of
its own. Each directive rule names the directive and the component, where its name and label
go, which attributes it keeps, what it requires and whether adjacent elements are grouped. Code rules (`codes`) do the same for fenced code: a block becomes a component by its language or a meta `key="value"`, or adjacent blocks are wrapped together, as tabbed code is. A container rule's `"treeAs"` reads its lists as a file tree, as Starlight's `<FileTree>` reads one, into a JSON attribute of items `{ name, type, comment?, highlighted?, children? }`: a name ending in `/` or holding a list is a directory, a **bold** name is highlighted, text after the name is its comment and `…` stands for more files.
Run it after `md.directive` and before `md.to_html` and `mdx.compile`, which compile the
elements to component calls.

```python
from textlane import md, textlane

rule = {"name": "note", "component": "Callout", "nameAs": "type", "labelAs": "title"}
file = (
    textlane()
    .use(md.parse())
    .use(md.directive())
    .use(md.mdx())
    .use(md.component({"directives": [rule]}))
    .use(md.stringify())
    .process(":::note[Heads up]\nRead *this*.\n:::")
)
file["value"]
# <Callout type="note" title="Heads up">
#   Read *this*.
# </Callout>
```

<!-- generated:options md.component -->

| Option              | Type                  | Default | Description                                                            |
| ------------------- | --------------------- | ------- | ---------------------------------------------------------------------- |
| `directives`        | `List[DirectiveRule]` |         | Rules for directives; the first that matches a directive converts it.  |
| `alerts`            | `Optional[AlertRule]` |         | GitHub alerts, `> [!NOTE]`, into a component.                          |
| `nodes`             | `Dict[str, str]`      | `{}`    | Node types, such as `defList`, into components keeping their children. |
| `unknownDirectives` | `UnknownDirectives`   |         | What becomes of a directive no rule converts.                          |
| `codes`             | `List[CodeRule]`      |         | Rules for fenced code; the first that matches a block applies.         |

**`DirectiveRule`**

How `md.component` turns matching directives into one component.

| Option         | Type                            | Default | Description                                                                                                                                                           |
| -------------- | ------------------------------- | ------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `name`         | `Names`                         |         | The directive's name, or several.                                                                                                                                     |
| `kind`         | `Optional[DirectiveKind]`       |         | The kind of directive: `text`, `leaf` or `container`; any when left out.                                                                                              |
| `where`        | `Dict[str, str]`                | `{}`    | Attribute values the directive must have.                                                                                                                             |
| `component`    | `str`                           |         | The component: a text element for a text directive, else a flow element.                                                                                              |
| `nameAs`       | `Optional[str]`                 | `None`  | The attribute that gets the directive's name.                                                                                                                         |
| `labelAs`      | `Optional[str]`                 | `None`  | The attribute that gets the label's text: a container's label paragraph or a text or leaf directive's content, which is then dropped.                                 |
| `labelDefault` | `Optional[str]`                 | `None`  | The label's text when there is none.                                                                                                                                  |
| `attributes`   | `Optional[List[AttributeRule]]` |         | The attributes to keep, in order; every one, as written, when left out.                                                                                               |
| `required`     | `List[str]`                     | `[]`    | Attributes without which the directive is left to `unknownDirectives`.                                                                                                |
| `group`        | `Optional[GroupRule]`           |         | Wrap runs of adjacent siblings this rule converted in another component.                                                                                              |
| `treeAs`       | `Optional[str]`                 | `None`  | The attribute that gets the lists of a container directive as a file tree, JSON items `{name, type, comment?, highlighted?, children?}`; the lists leave its content. |

**`AlertRule`**

How `md.component` turns GitHub alerts, a block quote opening with `[!TYPE]`, into a component.

| Option      | Type             | Default | Description                                                                           |
| ----------- | ---------------- | ------- | ------------------------------------------------------------------------------------- |
| `component` | `str`            |         | The component an alert becomes.                                                       |
| `typeAs`    | `str`            |         | The attribute that gets the alert's type.                                             |
| `types`     | `Dict[str, str]` |         | Each alert type, as written, to the value of `typeAs`; other types stay block quotes. |

**`UnknownDirectives`**

What `md.component` does with a directive no rule converts.

Takes `Union[Literal["keep"], Literal["source"]]`.

**`CodeRule`**

How `md.component` turns matching code blocks into a component, or groups them.

| Option       | Type                      | Default | Description                                                                      |
| ------------ | ------------------------- | ------- | -------------------------------------------------------------------------------- |
| `lang`       | `Optional[Names]`         |         | The block's language, or several; any when left out.                             |
| `meta`       | `Optional[str]`           | `None`  | A `key="value"` the block's meta must hold.                                      |
| `component`  | `Optional[str]`           | `None`  | The component the block becomes; left out, the block stays code and only groups. |
| `valueAs`    | `Optional[str]`           | `None`  | The attribute that gets the code; left out, the code is the component's text.    |
| `attributes` | `List[AttributeRule]`     |         | Meta `key="value"` pairs to keep as attributes, in order.                        |
| `group`      | `Optional[CodeGroupRule]` |         | Wrap runs of adjacent blocks this rule matched in another component.             |

**`Names`**

One name or several.

Takes `Union[str, List[str]]`.

**`DirectiveKind`**

A directive's kind, by how many colons open it.

Takes `Literal["text", "leaf", "container"]`.

**`AttributeRule`**

An attribute to keep: its name, or its name with what to call it and a default.

Takes `Union[str, Inline1]`.

**`GroupRule`**

The component wrapping a run of adjacent siblings one rule converted.

| Option      | Type            | Default | Description                                                 |
| ----------- | --------------- | ------- | ----------------------------------------------------------- |
| `component` | `str`           |         | The wrapping component.                                     |
| `min`       | `Optional[int]` | `None`  | The shortest run wrapped; 2 when left out.                  |
| `inherit`   | `List[str]`     | `[]`    | Attributes the wrapper copies from the run's first element. |

**`CodeGroupRule`**

The component wrapping a run of adjacent code blocks one rule matched; the meta keys it reads are taken out of each block's meta.

| Option      | Type            | Default | Description                                                                        |
| ----------- | --------------- | ------- | ---------------------------------------------------------------------------------- |
| `component` | `str`           |         | The wrapping component.                                                            |
| `min`       | `Optional[int]` | `None`  | The shortest run wrapped; 2 when left out.                                         |
| `collect`   | `Optional[str]` | `None`  | The attribute that gets the run's values of the rule's `meta` key, as a JSON list. |
| `inherit`   | `List[str]`     | `[]`    | Meta keys the wrapper copies from the run's first block as attributes.             |

<!-- /generated -->

## md.packageCommand

Turns a fenced block of package-manager commands into one MDX JSX element with the commands
converted for each manager, as `@docusaurus/remark-plugin-npm2yarn` does, so a site's install
tabs need no JavaScript plugin. The block's language names the ecosystem: `npm` converts as
the npm-to-yarn package does, for pnpm, yarn, bun, deno and nub; `pip` for uv, poetry and pdm;
`cargo` for cargo-binstall. Each manager becomes a string attribute named after it, in the
order `ecosystems` lists them, and each line converts on its own; a line a manager has no
equivalent for stays as written. Run it before `md.to_html` and `mdx.compile`.

````python
from textlane import md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.mdx())
    .use(md.package_command())
    .use(md.stringify())
    .process("```npm\nnpm install textlane\n```")
)
file["value"]
# <PackageTabs npm="npm install textlane" pnpm="pnpm add textlane" yarn="yarn add textlane" bun="bun add textlane" />
````

<!-- generated:options md.packageCommand -->

Follows [npm-to-yarn](https://www.npmjs.com/package/npm-to-yarn).

| Option       | Type         | Default         | Description                                                                          |
| ------------ | ------------ | --------------- | ------------------------------------------------------------------------------------ |
| `component`  | `str`        | `"PackageTabs"` | The component a block becomes; one string attribute per manager.                     |
| `ecosystems` | `Ecosystems` |                 | Each ecosystem's managers, in attribute order; an empty list leaves its blocks code. |

**`Ecosystems`**

The managers `md.packageCommand` converts each ecosystem's commands for.

| Option  | Type                 | Default | Description                                                                                       |
| ------- | -------------------- | ------- | ------------------------------------------------------------------------------------------------- |
| `npm`   | `List[NpmManager]`   |         | Managers for `npm` blocks, converted as npm-to-yarn converts; npm, pnpm, yarn and bun by default. |
| `pip`   | `List[PipManager]`   |         | Managers for `pip` blocks; pip, uv, poetry and pdm by default.                                    |
| `cargo` | `List[CargoManager]` |         | Managers for `cargo` blocks; cargo and cargo-binstall by default.                                 |

**`NpmManager`**

A JavaScript package manager an `npm` command converts to.

Takes `Literal["npm", "pnpm", "yarn", "bun", "deno", "nub"]`.

**`PipManager`**

A Python package manager a `pip` command converts to.

Takes `Literal["pip", "uv", "poetry", "pdm"]`.

**`CargoManager`**

A Rust installer a `cargo` command converts to.

Takes `Literal["cargo", "cargo-binstall"]`.

<!-- /generated -->

## md.headingId

Reads `{#id}` at the end of a heading, after a space, as its id, as remark-heading-id does; `md.toHtml` puts it on the heading. remark-heading-id's `defaults` is not ported: `html.slug` gives the other headings ids. With `{"attributes": True}` it reads a whole attribute list instead, as `md.directive` reads one: `## API {#api .wide data-step=2}` gives the heading an id, classes and a `data-step`. Only the last braces are read, event handlers (`on…`) and invalid names are dropped, and on untrusted input the result still belongs behind `html.sanitize`. `"markers"` turns literal heading suffixes into attributes and strips them, before or after an attribute list: `{"[step]": {"data-step": true}, "[!toc]": {"data-toc-hidden": true}}` reads `## Install [step]` as `<h2 data-step>Install</h2>`.

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.heading_id())
    .use(md.to_html())
    .use(html.stringify())
    .process("# Install {#setup}")
)
file["value"]
# <h1 id="setup">Install</h1>
```

<!-- generated:options md.headingId -->

Follows [remark-heading-id](https://www.npmjs.com/package/remark-heading-id).

| Option       | Type                                | Default | Description                                                                                                                                                           |
| ------------ | ----------------------------------- | ------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `attributes` | `bool`                              | `False` | Read a whole attribute list, `{#id .class key=value key="v w"}`, as directives write theirs: classes become `className` and other keys properties.                    |
| `markers`    | `Dict[str, Dict[str, MarkerValue]]` |         | Literal suffixes, such as `[step]`, stripped from a heading's end for the attributes each names, a value or `true` for a bare one: `{"[step]": {"data-step": true}}`. |

**`MarkerValue`**

An attribute a heading marker sets: its value, or `true` for the bare attribute.

Takes `Union[bool, str]`.

<!-- /generated -->

## md.pathRewrite

Rewrites the URLs of images, links and definitions as a site serves them, from where the document lives: a relative URL is resolved against the file's path and made relative to `root`, every root-relative URL gets `base` once, and a link to a Markdown file becomes its page's URL, from `pages` or by `markdownLinks` (`keep`, `directory` or `html`). URLs with a scheme, protocol-relative ones and fragments stay as they are. Pass the document's path with the text.

```python
from textlane import html, md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.path_rewrite({"root": "/repo/docs", "base": "/site", "markdownLinks": "directory"}))
    .use(md.to_html())
    .use(html.stringify())
    .process({
        "value": "![logo](../logo.png) and [next](./next.md)",
        "path": "/repo/docs/guide/intro.md",
    })
)
file["value"]
# <p><img src="/site/logo.png" alt="logo"> and <a href="/site/guide/next/">next</a></p>
```

<!-- generated:options md.pathRewrite -->

| Option          | Type                      | Default | Description                                                                                                                   |
| --------------- | ------------------------- | ------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `root`          | `Optional[str]`           | `None`  | The directory URLs count from, as the file's path spells it; the path's own starting point when left out.                     |
| `base`          | `Optional[str]`           | `None`  | The URL the site is served at, such as `/docs`, put before every root-relative URL once.                                      |
| `nodes`         | `Optional[List[UrlNode]]` |         | The nodes whose URLs are rewritten; images, links and definitions when left out.                                              |
| `pages`         | `Dict[str, str]`          | `{}`    | Markdown files, relative to `root`, to the URL of their page; a link to one of them takes it, before `markdownLinks` applies. |
| `markdownLinks` | `MarkdownLinks`           |         | How a link to a Markdown file no page names becomes a URL.                                                                    |

**`UrlNode`**

A node with a URL.

Takes `Literal["image", "link", "definition"]`.

**`MarkdownLinks`**

How a link to a Markdown file becomes a URL.

Takes `Union[Literal["keep"], Literal["directory"], Literal["html"]]`.

<!-- /generated -->

## md.shiftHeading

Moves every heading's depth by `shift`, kept within 1 to 6, as rehype-shift-heading does for hast: `-1` makes each h2 an h1. `dropTitle` first drops the document's first node when it is an h1, looking past frontmatter and ESM, which suits a page whose title is rendered on its own; dropping before shifting keeps a document that opens on an h2.

```python
from textlane import md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.shift_heading({"shift": -1, "dropTitle": True}))
    .use(md.stringify())
    .process("# 0.2.0\n\n### Features\n\n- a")
)
file["value"]
# ## Features
#
# * a
```

<!-- generated:options md.shiftHeading -->

Follows [rehype-shift-heading](https://www.npmjs.com/package/rehype-shift-heading).

| Option      | Type   | Default | Description                                                                        |
| ----------- | ------ | ------- | ---------------------------------------------------------------------------------- |
| `shift`     | `int`  | `0`     | The levels added to each heading's depth: `-1` makes an h2 an h1, `1` an h1 an h2. |
| `dropTitle` | `bool` | `False` | Drop the document's first node when it is an h1, looking past frontmatter and ESM. |

<!-- /generated -->

## md.strip

Strips mdast down to its text, as strip-markdown does: code, HTML, tables and definitions go, headings become paragraphs and formatting gives way to its content, which suits search indexes and plain-text summaries. `keep` leaves node types as they are and `remove` drops more.

```python
from textlane import md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.strip())
    .use(md.stringify())
    .process("# Hello *world*\n\nSee [the docs](/docs).")
)
file["value"]
# Hello world
#
# See the docs.
```

<!-- generated:options md.strip -->

Follows [strip-markdown](https://www.npmjs.com/package/strip-markdown).

| Option   | Type        | Default | Description                                             |
| -------- | ----------- | ------- | ------------------------------------------------------- |
| `keep`   | `List[str]` | `[]`    | Node types to leave as they are, such as `inlineCode`.  |
| `remove` | `List[str]` | `[]`    | Node types to drop, besides those strip-markdown drops. |

<!-- /generated -->

## md.toc

Generates a table of contents, as remark-toc does: under the first heading matching `heading` (`Contents`, `Table of contents` or `TOC` by default), it replaces whatever sat before the next heading as deep or shallower with a list linking to every heading after, by the ids github-slugger makes from their text, or the ids `md.headingId` stored. `heading` and `skip` are JavaScript regular expression sources matched against a heading's whole text, case-insensitively, run by Rust's `regex`: lookaround and back references are refused, and case folding is Unicode's rather than JavaScript's. `parents` takes unist-util-is tests written as JSON (a type, an object of fields, a list), not functions.

```python
from textlane import md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.toc())
    .use(md.stringify())
    .process("# Guide\n\n## Contents\n\n## Install\n\n### From npm\n\n## Use")
)
file["value"]
# # Guide
#
# ## Contents
#
# * [Install](#install)
#   * [From npm](#from-npm)
# * [Use](#use)
#
# ## Install
#
# ### From npm
#
# ## Use
```

<!-- generated:options md.toc -->

Follows [remark-toc](https://www.npmjs.com/package/remark-toc).

| Option     | Type                   | Default | Description                                                                                                                                                                    |
| ---------- | ---------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `heading`  | `Optional[str]`        | `None`  | The heading to put the table under, as JavaScript regular expression source matched against its whole text, case-insensitively; `(table[ -]of[ -])?contents?\|toc` when unset. |
| `maxDepth` | `Optional[int]`        | `None`  | The deepest heading rank to list, 1 to 6.                                                                                                                                      |
| `minDepth` | `Optional[int]`        | `None`  | The shallowest heading rank to list, 1 to 6.                                                                                                                                   |
| `skip`     | `Optional[str]`        | `None`  | Headings to leave out, as JavaScript regular expression source matched against their whole text, case-insensitively.                                                           |
| `tight`    | `Optional[bool]`       | `None`  | Keep the list compact, without blank lines between items; `true` when unset.                                                                                                   |
| `ordered`  | `Optional[bool]`       | `None`  | Number the list instead of bulleting it.                                                                                                                                       |
| `prefix`   | `Optional[str]`        | `None`  | What each link's fragment starts with, such as `user-content-`.                                                                                                                |
| `parents`  | `Optional[ParentTest]` |         | The parents whose headings are listed, as a unist-util-is test: a node type, an object of fields a parent must equal, or a list of these; only the root when unset.            |

**`ParentTest`**

A unist-util-is test of a node, written as JSON: one check, or a list any of which passes.

Takes `Union[ParentCheck, List[ParentCheck]]`.

**`ParentCheck`**

One unist-util-is check: a node type, or fields a node must equal.

Takes `Union[str, Dict[str, Any]]`.

<!-- /generated -->

## md.stringify

Serializes mdast back into Markdown; its options pick the markers and spacing it writes.

```python
from textlane import md, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.stringify({"bullet": "-", "emphasis": "_"}))
    .process("* *one*\n* two")
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
    .process("Hi <kbd>Ctrl</kbd>")
)
file["value"]
# <p>Hi <kbd>Ctrl</kbd></p>
```

`"taskListLabels": {"checked": …, "unchecked": …}` gives each GFM task-list checkbox an `aria-label`, so screen readers name it; remark-rehype writes none, so it is off by default.

`"breaks": True` turns each line ending in text into a `<br>`, as remark-breaks does, so a single newline breaks the line as in GitHub comments.

<!-- generated:options md.toHtml -->

Follows [remark-rehype](https://www.npmjs.com/package/remark-rehype).

| Option                 | Type                       | Default | Description                                                                                                               |
| ---------------------- | -------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------- |
| `allowDangerousHtml`   | `bool`                     | `False` | Keep raw HTML in the tree instead of dropping it.                                                                         |
| `breaks`               | `bool`                     | `False` | Turn each line ending in text into a `<br>`, as remark-breaks does.                                                       |
| `clobberPrefix`        | `Optional[str]`            | `None`  | What footnote `id`s start with, so they cannot clobber the page's; `user-content-` when unset.                            |
| `footnoteBackContent`  | `Optional[str]`            | `None`  | The text of each footnote's back reference; `↩`, numbered from the second, when unset.                                    |
| `footnoteBackLabel`    | `Optional[str]`            | `None`  | The `aria-label` of each footnote's back reference; `Back to reference 1` and so on when unset.                           |
| `footnoteLabel`        | `Optional[str]`            | `None`  | The heading of the footnotes section; `Footnotes` when unset.                                                             |
| `footnoteLabelTagName` | `Optional[str]`            | `None`  | The tag name of the footnotes section's heading; `h2` when unset.                                                         |
| `passThrough`          | `List[str]`                | `[]`    | Node types without a handler to copy into hast as they are, their children converted.                                     |
| `taskListLabels`       | `Optional[TaskListLabels]` |         | An `aria-label` for each task-list checkbox, so screen readers name it; none when left out, as remark-rehype writes none. |

**`TaskListLabels`**

The labels of task-list checkboxes.

| Option      | Type  | Default | Description                                           |
| ----------- | ----- | ------- | ----------------------------------------------------- |
| `checked`   | `str` |         | The label of a checked box, such as `Completed task`. |
| `unchecked` | `str` |         | The label of an unchecked box, such as `Task`.        |

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
    .process("It is *the the* best.")
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
