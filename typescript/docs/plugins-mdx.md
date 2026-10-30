---
title: MDX plugins
description: Every MDX plugin, its options and the unified package it follows.
path: /typescript/mdx-plugins/
icon: braces
order: 8
---

# MDX plugins

MDX is Markdown with JSX. Parse it with `md.parse` and `md.mdx`, turn it into hast with `md.toHtml`, then compile it to JavaScript with `mdx.compile`.

## mdx.compile

Compiles the hast of an MDX document into a JavaScript module whose default export renders it.

```js
import { textlane } from "textlane"
import * as md from "textlane/md"
import * as mdx from "textlane/mdx"

const file = textlane()
  .use(md.parse())
  .use(md.mdx())
  .use(md.toHtml())
  .use(mdx.compile())
  .processSync("# Hi {name}")
file.value
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
