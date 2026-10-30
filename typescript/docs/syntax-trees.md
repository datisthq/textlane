---
title: Syntax trees
description: The mdast, hast and nlcst trees a pipeline passes between its stages.
path: /typescript/syntax-trees/
icon: network
order: 10
---

# Syntax trees

A pipeline passes one of three trees between its stages, each a [unist](https://github.com/syntax-tree/unist)
tree as the unified ecosystem defines it:

- [mdast](https://github.com/syntax-tree/mdast) — Markdown, from `md.parse`
- [hast](https://github.com/syntax-tree/hast) — HTML, from `html.parse` and `md.toHtml`
- [nlcst](https://github.com/syntax-tree/nlcst) — natural language, from `nl.parse`,
  `md.toNaturalLanguage` and `html.toNaturalLanguage`

`parse` returns the tree as plain JSON, node for node what the unified ecosystem produces,
and `run` and `stringify` take it back. Every node has a `type`; parents have `children`;
literals have a `value`.

```js
import { textlane } from "textlane"
import * as md from "textlane/md"

const tree = textlane().use(md.parse()).parse("# Hi *there*")
const heading = tree.children[0]
heading.type
// heading
heading.depth
// 1
heading.children[1].type
// emphasis
```

## Positions

Each node parsed from text carries a `position`: its `start` and `end` points, each with a
1-indexed `line` and `column` and a 0-indexed `offset`.

```js
import { textlane } from "textlane"
import * as md from "textlane/md"

const tree = textlane().use(md.parse()).parse("# Hi *there*")
tree.children[0].children[1].position.start
// {"line":1,"column":6,"offset":5}
```

## hast and nlcst

hast elements carry a `tagName` and `properties`, named as the DOM names them; nlcst nests
paragraphs, sentences and words:

```js
import { textlane } from "textlane"
import * as html from "textlane/html"
import * as nl from "textlane/nl"

const element = textlane()
  .use(html.parse({ fragment: true }))
  .parse('<p class="note">Hi</p>').children[0]
element.properties
// {"className":["note"]}
const sentence = textlane().use(nl.parse()).parse("Hi there.").children[0].children[0]
sentence.children.map(node => node.type).join(" ")
// WordNode WhiteSpaceNode WordNode PunctuationNode
```

A tree changed in JavaScript goes back through `run` or `stringify`, which read it as the
kind of tree the next stage takes.
