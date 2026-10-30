---
title: Errors
description: The error codes textlane returns and what each one means.
path: /rust/errors/
icon: circle-alert
order: 12
---

# Errors

Every fallible call returns `textlane::Error`, with a `code` naming what went wrong and a
`message` for people; it implements `std::error::Error`.

```rust
use textlane::Pipeline;

let error = Pipeline::new()
    .use_value(&serde_json::json!("md.nope"), None)
    .err()
    .ok_or("no error")?;
error.code
// unknown-plugin
```

| Code                              | When                                                                      |
| --------------------------------- | ------------------------------------------------------------------------- |
| `invalid-stage`                   | `freeze` finds a stage that does not take the tree the one before returns |
| `missing-parser`                  | `parse` runs with no parser in the processor                              |
| `async-in-sync`                   | `run_sync` meets a stage with asynchronous work                           |
| `unknown-node`                    | a compiler meets a node type it cannot write                              |
| `mdx-failed`                      | `mdx::Compile` meets JavaScript it cannot compile                         |
| `unsafe-number`                   | `nl::IndefiniteArticle` meets a number too large to read                  |
| `highlight-failed`                | `html::Highlight` cannot color a code block                               |
| `highlight-unavailable`           | `html::Highlight`'s grammars and themes failed to load                    |
| `missing-natural-language-parser` | `html::ToNaturalLanguage` runs in a build without the `nl` feature        |

`Pipeline` adds the codes of unified's API, as the Node.js and Python packages raise them:

| Code               | When                                                                     |
| ------------------ | ------------------------------------------------------------------------ |
| `unknown-plugin`   | `use_value` names a plugin textlane does not have                        |
| `invalid-plugin`   | `use_value` gets something that is neither a plugin, a list nor a preset |
| `invalid-options`  | a plugin's options do not match its types, when the pipeline freezes     |
| `invalid-data`     | `replace_data` gets something that is not an object                      |
| `frozen-processor` | `use_value` or a data setter is called on a frozen pipeline              |
| `missing-compiler` | `stringify` or `process` runs with no compiler in the pipeline           |
| `invalid-tree`     | a JSON tree is not valid unist                                           |
| `unsupported-node` | a JSON tree holds a node with no `type`                                  |
| `tree-too-deep`    | a tree nests too deep to write as JSON                                   |

Syntax errors in MDX carry the upstream's source and rule as their code, as
`micromark-extension-mdx-expression:acorn` for an expression that does not parse.
