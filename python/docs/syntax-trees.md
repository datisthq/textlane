---
title: Syntax trees
description: The mdast, hast and nlcst trees a pipeline passes between its stages.
path: /python/syntax-trees/
icon: network
order: 10
---

# Syntax trees

A pipeline passes one of three trees between its stages, each a [unist](https://github.com/syntax-tree/unist)
tree as the unified ecosystem defines it:

- [mdast](https://github.com/syntax-tree/mdast) — Markdown, from `md.parse`
- [hast](https://github.com/syntax-tree/hast) — HTML, from `html.parse` and `md.to_html`
- [nlcst](https://github.com/syntax-tree/nlcst) — natural language, from `nl.parse`,
  `md.to_natural_language` and `html.to_natural_language`

`parse` returns the tree as dicts and lists, node for node what the unified ecosystem
produces, and `run` and `stringify` take it back. Every node has a `type`; parents have
`children`; literals have a `value`. Keys keep their JavaScript names, such as `tagName`.

## Types

The parser types the tree: after `md.parse()`, `html.parse()` or `nl.parse()`, `parse`
returns the `Root` of `textlane.mdast`, `textlane.hast` or `textlane.nlcst`, `TypedDict`s
named node for node as [`@types/mdast`](https://www.npmjs.com/package/@types/mdast),
`@types/hast` and `@types/nlcst` name them, with `Point` and `Position` in `textlane.unist`.
They include the nodes textlane's syntax extensions add (`ContainerDirective`, `Math`,
`DefListNode`, `MdxJsxFlowElement`, `MdxjsEsm`, …), and MDX's `data.estree` is typed by
`textlane.estree`. Narrow a union by comparing `type`, and read an optional key, such as
`position` or `lang`, with `get`:

````python
from textlane import md, mdast, textlane

tree: mdast.Root = textlane().use(md.parse()).parse("```py\nx = 1\n```")
for node in tree["children"]:
    if node["type"] == "code":
        position = node.get("position")
        print(node.get("lang"), node["value"], position and position["start"]["line"])
# py x = 1 1
````

Before a parser is used, and for a tree `run` returns, the tree is a plain dict. `run` and
`stringify` take any mapping, so a typed tree goes back without a cast.

```python
from textlane import md, textlane

tree = textlane().use(md.parse()).parse("# Hi *there*")
heading = tree["children"][0]
heading["type"]
# heading
heading["depth"]
# 1
heading["children"][1]["type"]
# emphasis
```

## Positions

Each node parsed from text carries a `position`: its `start` and `end` points, each with a
1-indexed `line` and `column` and a 0-indexed `offset`.

```python
from textlane import md, textlane

tree = textlane().use(md.parse()).parse("# Hi *there*")
tree["children"][0]["children"][1]["position"]["start"]
# {'line': 1, 'column': 6, 'offset': 5}
```

## hast and nlcst

hast elements carry a `tagName` and `properties`, named as the DOM names them; nlcst nests
paragraphs, sentences and words:

```python
from textlane import html, nl, textlane

fragment = textlane().use(html.parse({"fragment": True}))
element = fragment.parse('<p class="note">Hi</p>')["children"][0]
element["properties"]
# {'className': ['note']}
sentence = textlane().use(nl.parse()).parse("Hi there.")["children"][0]["children"][0]
" ".join(node["type"] for node in sentence["children"])
# WordNode WhiteSpaceNode WordNode PunctuationNode
```

A tree changed in Python goes back through `run` or `stringify`, which read it as the kind
of tree the next stage takes.
