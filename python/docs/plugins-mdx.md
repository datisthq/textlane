---
title: MDX plugins
description: Every MDX plugin, its options and the unified package it follows.
path: /python/mdx-plugins/
icon: braces
order: 8
---

# MDX plugins

MDX is Markdown with JSX. Parse it with `md.parse` and `md.mdx`, turn it into hast with `md.toHtml`, then compile it to JavaScript with `mdx.compile`.

## mdx.compile

Compiles the hast of an MDX document into a JavaScript module whose default export renders it.

```python
from textlane import md, mdx, textlane

file = (
    textlane()
    .use(md.parse())
    .use(md.mdx())
    .use(md.to_html())
    .use(mdx.compile())
    .process("# Hi {name}")
)
file["value"]
# import {jsx as _jsx, jsxs as _jsxs} from "react/jsx-runtime";
# function _createMdxContent(props) {
#   const _components = {
#     h1: "h1",
#     ...props.components
# …
```

<!-- generated:options mdx.compile -->

Follows [@mdx-js/mdx](https://www.npmjs.com/package/@mdx-js/mdx).

| Option                     | Type             | Default | Description                                                                                                            |
| -------------------------- | ---------------- | ------- | ---------------------------------------------------------------------------------------------------------------------- |
| `jsx`                      | `bool`           | `False` | Keep JSX in the output instead of compiling it to function calls.                                                      |
| `jsxRuntime`               | `Optional[str]`  | `None`  | The JSX runtime: `automatic` (default) or `classic`.                                                                   |
| `jsxImportSource`          | `Optional[str]`  | `None`  | Where the automatic runtime imports `jsx` and `jsxs` from (default `react`).                                           |
| `pragma`                   | `Optional[str]`  | `None`  | The classic runtime's element function (default `React.createElement`).                                                |
| `pragmaFrag`               | `Optional[str]`  | `None`  | The classic runtime's fragment (default `React.Fragment`).                                                             |
| `pragmaImportSource`       | `Optional[str]`  | `None`  | Where the classic runtime imports the pragma's first name from (default `react`).                                      |
| `providerImportSource`     | `Optional[str]`  | `None`  | Where `useMDXComponents` is imported from, to take components from a provider.                                         |
| `outputFormat`             | `Optional[str]`  | `None`  | `program` (default), a module, or `function-body`, the body of a function that gets the runtime as its first argument. |
| `elementAttributeNameCase` | `Optional[str]`  | `None`  | How attribute names of HTML elements are cased: `react` (default) or `html`.                                           |
| `stylePropertyNameCase`    | `Optional[str]`  | `None`  | How style property names are cased: `dom` (default) or `css`.                                                          |
| `tableCellAlignToStyle`    | `Optional[bool]` | `None`  | Turn `align` on table cells into a `textAlign` style (default `true`).                                                 |

<!-- /generated -->
