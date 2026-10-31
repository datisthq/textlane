import { join, resolve } from "node:path"
import { compile } from "@mdx-js/mdx"
import { parse } from "acorn"
import type { Comment } from "acorn"
import { writeGroups } from "../actions/group/write.ts"

const TESTS = resolve(import.meta.dirname, "..", "tests")
const REPOSITORY = "https://github.com/syntax-tree/hast-util-to-estree"

// MDX nodes whose JavaScript a plugin stored as `data.estree`, the only place
// hast-util-to-estree reads it from; each tree is compiled by the pinned @mdx-js/mdx
const CASES: { description: string; tree: object }[] = [
  {
    description: "ESM built as estree only, as remark-mdx-frontmatter builds it",
    tree: root([esm("", program('export const frontmatter = {"title": "Hi"}', false))]),
  },
  {
    description: "ESM whose estree differs from its value",
    tree: root([esm("export const a = 1", program("export const b = 2", false))]),
  },
  {
    description: "ESM estree with positions and comments",
    tree: root([
      esm("", program("// lead\nexport const a = 1 /* trail */", true)),
      element("p", [text("x")]),
    ]),
  },
  {
    description: "ESM estree with offsets past any document",
    tree: root([
      esm("", { ...program("export const a = 1", false), start: 0, end: 1e300 }),
    ]),
  },
  {
    description: "flow expression built as estree only",
    tree: root([
      { type: "mdxFlowExpression", value: "", data: { estree: program("1 + 1", false) } },
    ]),
  },
  {
    description: "attribute value and spread built as estree only",
    tree: root([
      {
        type: "mdxJsxFlowElement",
        name: "Card",
        attributes: [
          {
            type: "mdxJsxAttribute",
            name: "count",
            value: {
              type: "mdxJsxAttributeValueExpression",
              value: "",
              data: { estree: program("2 * 3", false) },
            },
          },
          {
            type: "mdxJsxExpressionAttribute",
            value: "",
            data: { estree: program("({...props})", false) },
          },
        ],
        children: [],
      },
    ]),
  },
]

const tests = []
for (const { description, tree } of CASES) {
  const file = await compile("", { rehypePlugins: [() => () => structuredClone(tree)] })
  tests.push({ description, tree, expected: { js: String(file) } })
}
writeGroups({
  path: join(TESTS, "mdx", "compile-estree.json"),
  repository: REPOSITORY,
  groups: [
    {
      description: "hast-util-to-estree MDX nodes with data.estree",
      source: { repository: REPOSITORY, version: "3.1.3", license: "MIT" },
      pipeline: [["mdx.compile"]],
      tests,
    },
  ],
})
console.log(`estree: ${tests.length} cases`)

// acorn's program as micromark's MDX extensions store it; without positions, as
// estree-util-value-to-estree builds one
function program(source: string, positions: boolean) {
  const comments: Comment[] = []
  const tree = parse(source, {
    ecmaVersion: "latest",
    sourceType: "module",
    onComment: comments,
  })
  const estree = { ...tree, comments }
  return JSON.parse(
    JSON.stringify(estree, (key, value) =>
      positions || (key !== "start" && key !== "end") ? value : undefined,
    ),
  )
}

function root(children: object[]) {
  return { type: "root", children }
}

function esm(value: string, estree: object) {
  return { type: "mdxjsEsm", value, data: { estree } }
}

function element(tagName: string, children: object[]) {
  return { type: "element", tagName, properties: {}, children }
}

function text(value: string) {
  return { type: "text", value }
}
