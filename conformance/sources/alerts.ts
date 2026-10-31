import { readFileSync } from "node:fs"
import { stripTypeScriptTypes } from "node:module"
import { join, resolve } from "node:path"
import { parse } from "acorn"
import type { AnyNode } from "acorn"
import { simple } from "acorn-walk"
import rehypeStringify from "rehype-stringify"
import { remarkAlert } from "remark-github-blockquote-alert"
import remarkGfm from "remark-gfm"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import { unified } from "unified"
import { writeGroups } from "../actions/group/write.ts"
import { evaluateNode, isObject } from "../actions/node/evaluate.ts"
import { fetchSource } from "../actions/source/fetch.ts"

const OUTPUT = join(resolve(import.meta.dirname, "..", "tests", "md"), "gfm-alerts.json")
const ALERT = {
  url: "https://github.com/jaywcjlove/remark-github-blockquote-alert",
  commit: "19c51161a3a9ff8f6d4ec649439a2e475f0e8170",
}
// Inputs of textlane's own: alerts after other blocks, lowercase types, unknown types,
// nested quotes and a marker with text after it on its line
const OWN = [
  "> # Heading\n> [!tip]\n> Advice.",
  "> [!NOTE] Extra words\n> Body.",
  "> [!NOTE] Extra words",
  "> [!UNKNOWN]\n> Body.",
  "> [!WARNING]\n> > [!CAUTION]\n> > Nested.",
  "> Text\n>\n> [!IMPORTANT]\n> Later paragraph.",
  "- > [!NOTE]\n  > In a list.",
  "> [!NOTE]\n> See https://example.com and ~~this~~.",
]

// `md.gfm({ alerts })` stands in for remark-gfm and remark-github-blockquote-alert; each
// test's `markdown` and options render through the pinned packages, upstream's `html` kept
// where it differs
const groups = new Map<string, { options: Record<string, unknown>; tests: object[] }>()
let written = 0
for (const { input, options, html } of readTests()) addCase(input, options, html)
for (const input of OWN) addCase(input, {}, undefined)
writeGroups({
  path: OUTPUT,
  repository: ALERT.url,
  groups: [...groups].map(([key, group]) => ({
    description: `remark-github-blockquote-alert${key === "{}" ? "" : ` ${key}`}`,
    source: {
      repository: ALERT.url,
      commit: ALERT.commit,
      path: "test/index.test.ts",
      license: "MIT",
    },
    pipeline: [
      ["md.parse"],
      [
        "md.gfm",
        { alerts: Object.keys(group.options).length > 0 ? group.options : true },
      ],
      ["md.toHtml"],
      ["html.stringify"],
    ],
    tests: group.tests,
  })),
})
console.log(`alerts: ${written} cases`)

function addCase(
  input: string,
  options: Record<string, unknown>,
  upstream: string | undefined,
) {
  const output = String(
    unified()
      .use(remarkParse)
      .use(remarkGfm)
      .use(remarkAlert, options)
      .use(remarkRehype)
      .use(rehypeStringify)
      .processSync(input),
  )
  const key = JSON.stringify(options)
  const group = groups.get(key) ?? { options, tests: [] }
  const drift =
    upstream === undefined || upstream === output
      ? {}
      : { comment: `upstream expects: ${upstream}` }
  group.tests.push({
    description: JSON.stringify(input),
    ...drift,
    input,
    expected: { html: output },
  })
  groups.set(key, group)
  written += 1
}

// Each `it(…)` holds `const markdown = …`, often `const html = …`, and one
// `.use(remarkAlert…, options?)`
function readTests() {
  const root = fetchSource({ repository: ALERT.url, commit: ALERT.commit })
  const source = stripTypeScriptTypes(
    readFileSync(join(root, "test", "index.test.ts"), "utf8"),
  )
  const program = parse(source, { ecmaVersion: "latest", sourceType: "module" })
  const tests: {
    input: string
    options: Record<string, unknown>
    html: string | undefined
  }[] = []
  simple(program, {
    CallExpression(node) {
      if (node.callee.type !== "Identifier" || node.callee.name !== "it") return
      const body = node.arguments[1]
      if (body?.type !== "ArrowFunctionExpression" && body?.type !== "FunctionExpression")
        return
      const constants = new Map<string, AnyNode>()
      let options: unknown = {}
      simple(body, {
        VariableDeclarator(declarator) {
          if (declarator.id.type === "Identifier" && declarator.init)
            constants.set(declarator.id.name, declarator.init)
        },
        CallExpression(call) {
          if (
            call.callee.type === "MemberExpression" &&
            call.callee.property.type === "Identifier" &&
            call.callee.property.name === "use" &&
            call.arguments[0]?.type === "Identifier" &&
            call.arguments[0].name.startsWith("remarkAlert") &&
            call.arguments[1]
          )
            options = evaluateNode(call.arguments[1], new Map())
        },
      })
      const input = evaluateNode(constants.get("markdown"), new Map())
      const html = evaluateNode(constants.get("html"), new Map())
      if (typeof input !== "string" || !isObject(options)) return
      tests.push({
        input,
        options: { ...options },
        html: typeof html === "string" ? html : undefined,
      })
    },
  })
  return tests
}
