---
title: Errors
description: The error codes textlane throws and what each one means.
path: /typescript/errors/
icon: circle-alert
order: 12
---

# Errors

Textlane throws an `Error` whose `code` names what went wrong, so a caller can tell the
cases apart without reading the message:

```js
import { textlane } from "textlane"

let code = ""
try {
  textlane().use("md.nope")
} catch (error) {
  code = error.code
}
code
// unknown-plugin
```

| Code                    | When                                                                  |
| ----------------------- | --------------------------------------------------------------------- |
| `unknown-plugin`        | `use` names a plugin textlane does not have                           |
| `invalid-plugin`        | `use` gets something that is neither a plugin, a list nor a preset    |
| `invalid-options`       | a plugin's options do not match its types, when the pipeline is built |
| `invalid-data`          | `data(object)` gets something that is not an object                   |
| `frozen-processor`      | `use` or a `data` setter is called on a frozen processor              |
| `missing-parser`        | `parse` or `process` runs with no parser in the pipeline              |
| `missing-compiler`      | `stringify` or `process` runs with no compiler in the pipeline        |
| `invalid-stage`         | a stage gets a tree it does not take, such as hast before `md.toHtml` |
| `async-in-sync`         | `processSync` or `runSync` meets a stage with asynchronous work       |
| `invalid-tree`          | a tree given to `run` or `stringify` is not valid unist               |
| `unsupported-node`      | a tree given to `run` or `stringify` holds a node with no `type`      |
| `unknown-node`          | a compiler meets a node type it cannot write                          |
| `tree-too-deep`         | a tree nests too deep to pass between Rust and JavaScript             |
| `mdx-failed`            | `mdx.compile` meets JavaScript it cannot compile                      |
| `unsafe-number`         | `nl.indefiniteArticle` meets a number too large to read               |
| `highlight-failed`      | `html.highlight` cannot color a code block                            |
| `highlight-unavailable` | `html.highlight`'s grammars and themes failed to load                 |

Syntax errors in MDX carry the upstream's source and rule as their code, as
`micromark-extension-mdx-expression:acorn` for an expression that does not parse.

Options are checked when the pipeline is built, the first time it runs or freezes, not when
`use` is called.
