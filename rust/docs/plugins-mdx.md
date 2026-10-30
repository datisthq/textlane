---
title: MDX plugins
description: Every MDX plugin, its options and the unified package it follows.
path: /rust/mdx-plugins/
icon: braces
order: 8
---

# MDX plugins

MDX is Markdown with JSX. Parse it with `md.parse` and `md.mdx`, turn it into hast with `md.toHtml`, then compile it to JavaScript with `mdx.compile`.

The tables list each option as `Pipeline` takes it, by its JSON key and type; each plugin
struct's own fields and Rust types are on [docs.rs](https://docs.rs/textlane).

## mdx.compile

Compiles the hast of an MDX document into a JavaScript module whose default export renders it.

```rust
use textlane::{Processor, md, mdx};

let output = Processor::new()
    .with(md::Parse::default())
    .with(md::Mdx::default())
    .with(md::ToHtml::default())
    .with(mdx::Compile::default())
    .freeze()?
    .process("# Hi {name}")?;
output.value
// import {jsx as _jsx, jsxs as _jsxs} from "react/jsx-runtime";
// function _createMdxContent(props) {
//   const _components = {
//     h1: "h1",
//     ...props.components
// …
```

<!-- generated:options mdx.compile -->

Follows [@mdx-js/mdx](https://www.npmjs.com/package/@mdx-js/mdx).

| Option                     | Type              | Default | Description                                                                                                            |
| -------------------------- | ----------------- | ------- | ---------------------------------------------------------------------------------------------------------------------- |
| `jsx`                      | `boolean`         | `false` | Keep JSX in the output instead of compiling it to function calls.                                                      |
| `jsxRuntime`               | `string \| null`  | `null`  | The JSX runtime: `automatic` (default) or `classic`.                                                                   |
| `jsxImportSource`          | `string \| null`  | `null`  | Where the automatic runtime imports `jsx` and `jsxs` from (default `react`).                                           |
| `pragma`                   | `string \| null`  | `null`  | The classic runtime's element function (default `React.createElement`).                                                |
| `pragmaFrag`               | `string \| null`  | `null`  | The classic runtime's fragment (default `React.Fragment`).                                                             |
| `pragmaImportSource`       | `string \| null`  | `null`  | Where the classic runtime imports the pragma's first name from (default `react`).                                      |
| `providerImportSource`     | `string \| null`  | `null`  | Where `useMDXComponents` is imported from, to take components from a provider.                                         |
| `outputFormat`             | `string \| null`  | `null`  | `program` (default), a module, or `function-body`, the body of a function that gets the runtime as its first argument. |
| `elementAttributeNameCase` | `string \| null`  | `null`  | How attribute names of HTML elements are cased: `react` (default) or `html`.                                           |
| `stylePropertyNameCase`    | `string \| null`  | `null`  | How style property names are cased: `dom` (default) or `css`.                                                          |
| `tableCellAlignToStyle`    | `boolean \| null` | `null`  | Turn `align` on table cells into a `textAlign` style (default `true`).                                                 |

<!-- /generated -->
