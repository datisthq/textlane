import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { parse } from "acorn"
import type { AnyNode, CallExpression } from "acorn"
import { ancestor } from "acorn-walk"
import { writeGroups } from "../actions/group/write.ts"
import { isStressInput } from "../actions/input/check.ts"
import {
  createPipeline,
  parseMarkdown,
  renderMarkdown,
  stringifyTree,
} from "../actions/markdown/render.ts"
import { createMdastPipeline, extractMdastCases } from "../actions/mdast/extract.ts"
import type { MdastSyntax } from "../actions/mdast/extract.ts"
import { evaluateNode, isObject } from "../actions/node/evaluate.ts"
import { resolveConstants } from "../actions/node/scope.ts"
import { fetchSource } from "../actions/source/fetch.ts"
import { isAssertEqual } from "../actions/test/assert.ts"
import { findTestTitle } from "../actions/test/find.ts"

const OUTPUT = join(resolve(import.meta.dirname, "..", "tests", "md"), "math.json")
const MICROMARK = {
  url: "https://github.com/micromark/micromark-extension-math",
  commit: "4c9e82c46f4ee3bb794382b84f9fa537f7452f9e",
}
const MDAST = {
  url: "https://github.com/syntax-tree/mdast-util-math",
  commit: "23647664692ae59be25896d0099ba3f7157d6a76",
}
const REMARK = {
  url: "https://github.com/remarkjs/remark-math",
  commit: "76c14978ff6805011b8c56727c54104a511b9055",
}

// `md.math` options keep remark-math's names
const MATH: MdastSyntax = {
  plugin: "md.math",
  extension: "math",
  isExtension: name => name.startsWith("math"),
  options: new Set(["singleDollarTextMath"]),
}

type Source = { repository: string; commit: string; path: string; license: string }
type Group = { description: string; source: Source; pipeline: unknown[]; tests: object[] }
type Kind = "tree" | "html" | "md"

const skipped = new Map<string, number>()
const seen = new Set<string>()
const groups = new Map<string, Group>()
let written = 0
importMath()

function importMath() {
  for (const found of extractMicromarkInputs()) {
    addTreeOrRender({
      repository: MICROMARK,
      path: "test/index.js",
      name: "micromark-extension-math",
      kind: "tree",
      ...found,
    })
  }
  const mdastSource = readFileSync(
    join(fetchSource({ repository: MDAST.url, commit: MDAST.commit }), "test.js"),
    "utf8",
  )
  for (const found of extractMdastCases({
    source: mdastSource,
    syntax: MATH,
    countSkip,
  })) {
    const optionsKey = JSON.stringify({
      ...found.options.plugin,
      ...found.options.stringify,
    })
    const description = `mdast-util-math ${found.kind}${optionsKey === "{}" ? "" : ` ${optionsKey}`}`
    const source = {
      repository: MDAST.url,
      commit: MDAST.commit,
      path: "test.js",
      license: "MIT",
    }
    add({
      description,
      source,
      pipeline: createMdastPipeline({
        syntax: MATH,
        kind: found.kind,
        options: found.options,
      }),
      test: found.test,
    })
  }
  for (const found of extractRemarkCases()) {
    addTreeOrRender({
      repository: REMARK,
      path: "packages/remark-math/test.js",
      name: "remark-math",
      ...found,
    })
  }
  for (const repository of [MICROMARK, MDAST, REMARK]) {
    const own = [...groups.values()].filter(
      group => group.source.repository === repository.url,
    )
    writeGroups({ path: OUTPUT, repository: repository.url, groups: own })
  }
  console.log(`math: ${written} cases`)
  for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)
}

// micromark's math HTML embeds KaTeX's output, so only its inputs carry over, as tree cases;
// remark-math's HTML is the KaTeX-free mdast to hast mapping `md.toHtml` produces
function addTreeOrRender(props: {
  repository: { url: string; commit: string }
  path: string
  name: string
  kind: Kind
  title: string
  input: string
  math: Record<string, unknown>
}) {
  if (isStressInput(props.input)) return countSkip("a stress test")
  if (!props.input.isWellFormed())
    return countSkip("a lone surrogate, which UTF-8 input cannot hold")
  const math = withOptions("md.math", props.math)
  const pipeline =
    props.kind === "tree"
      ? [["md.parse"], math]
      : props.kind === "md"
        ? [["md.parse"], math, ["md.stringify"]]
        : createPipeline({ allowDangerousHtml: false, math: props.math })
  const expected =
    props.kind === "tree"
      ? { tree: parseMarkdown(props.input, { math: props.math }) }
      : props.kind === "md"
        ? {
            md: stringifyTree({
              tree: parseMarkdown(props.input, { math: props.math }),
              math: props.math,
            }),
          }
        : {
            html: renderMarkdown({
              allowDangerousHtml: false,
              math: props.math,
              markdown: props.input,
            }),
          }
  const optionsKey =
    Object.keys(props.math).length > 0 ? ` ${JSON.stringify(props.math)}` : ""
  const source = {
    repository: props.repository.url,
    commit: props.repository.commit,
    path: props.path,
    license: "MIT",
  }
  add({
    description: `${props.name} (${props.kind})${optionsKey}`,
    source,
    pipeline,
    test: { description: props.title, input: props.input, expected },
  })
}

function add(props: {
  description: string
  source: Source
  pipeline: unknown[]
  test: object
}) {
  const key = `${JSON.stringify(props.pipeline)}\u0000${JSON.stringify(props.test)}`
  if (seen.has(key)) return countSkip("duplicate of an earlier case")
  seen.add(key)
  const group = groups.get(props.description) ?? {
    description: props.description,
    source: props.source,
    pipeline: props.pipeline,
    tests: [],
  }
  group.tests.push(props.test)
  groups.set(props.description, group)
  written += 1
}

// `micromark(input, {extensions: [math(options?)], …})`
function extractMicromarkInputs() {
  const root = fetchSource({ repository: MICROMARK.url, commit: MICROMARK.commit })
  const program = parse(readFileSync(join(root, "test", "index.js"), "utf8"), {
    ecmaVersion: "latest",
    sourceType: "module",
  })
  const found: { title: string; input: string; math: Record<string, unknown> }[] = []
  const counts = new Map<string, number>()
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      if (!isAssertEqual(node.callee) && !isAssertThrows(node.callee)) return
      const call =
        node.arguments[0]?.type === "ArrowFunctionExpression" ||
        node.arguments[0]?.type === "FunctionExpression"
          ? findMicromarkCall(node.arguments[0])
          : node.arguments[0]
      if (call?.type !== "CallExpression" || !isCallTo(call, "micromark")) return
      const constants = resolveConstants({ program, ancestors })
      const input = evaluateNode(call.arguments[0], constants)
      if (typeof input !== "string") return countSkip("not statically evaluable")
      const math = readMicromarkMath(call.arguments[1], constants)
      if (typeof math === "string") return countSkip(math)
      const title = findTestTitle(ancestors, constants) ?? "micromark-extension-math"
      const count = (counts.get(title) ?? 0) + 1
      counts.set(title, count)
      found.push({ title: count === 1 ? title : `${title} (${count})`, input, math })
    },
  })
  return found
}

function findMicromarkCall(node: AnyNode): CallExpression | undefined {
  let call: CallExpression | undefined
  ancestor(node, {
    CallExpression(inner) {
      if (!call && isCallTo(inner, "micromark")) call = inner
    },
  })
  return call
}

function readMicromarkMath(
  node: AnyNode | undefined,
  constants: Map<string, AnyNode>,
): Record<string, unknown> | string {
  const target = node?.type === "Identifier" ? constants.get(node.name) : node
  if (target?.type !== "ObjectExpression") return "options not statically evaluable"
  for (const property of target.properties) {
    if (
      property.type !== "Property" ||
      property.key.type !== "Identifier" ||
      property.key.name !== "extensions"
    )
      continue
    if (property.value.type !== "ArrayExpression")
      return "options not statically evaluable"
    for (const element of property.value.elements) {
      if (element?.type !== "CallExpression" || !isCallTo(element, "math")) continue
      const options = element.arguments[0]
        ? evaluateNode(element.arguments[0], constants)
        : {}
      if (!isObject(options)) return "extension options not statically evaluable"
      const unknown = Object.keys(options).filter(key => !MATH.options.has(key))
      if (unknown.length > 0)
        return `an extension option without a pipeline equivalent (${unknown.join(", ")})`
      return { ...options }
    }
  }
  return "no math extension enabled"
}

// `unified().use(remarkParse).use(remarkMath, options?)[.use(…)].parse(md) | .processSync(md)`:
// the plugins used decide whether the case is a tree, HTML or Markdown
function extractRemarkCases() {
  const root = fetchSource({ repository: REMARK.url, commit: REMARK.commit })
  const program = parse(
    readFileSync(join(root, "packages", "remark-math", "test.js"), "utf8"),
    { ecmaVersion: "latest", sourceType: "module" },
  )
  const found: {
    kind: Kind
    title: string
    input: string
    math: Record<string, unknown>
  }[] = []
  const counts = new Map<string, number>()
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      if (
        node.callee.type !== "MemberExpression" ||
        node.callee.property.type !== "Identifier"
      )
        return
      const method = node.callee.property.name
      if (method !== "parse" && method !== "process" && method !== "processSync") return
      const constants = resolveConstants({ program, ancestors })
      const plugins = readChain(node.callee.object, constants)
      if (!plugins?.has("remarkMath")) return
      const input = evaluateNode(node.arguments[0], constants)
      if (typeof input !== "string") return countSkip("not statically evaluable")
      const math = plugins.get("remarkMath")
        ? evaluateNode(plugins.get("remarkMath"), constants)
        : {}
      if (!isObject(math)) return countSkip("options not statically evaluable")
      const kind: Kind =
        method === "parse"
          ? "tree"
          : plugins.has("remarkRehype")
            ? "html"
            : plugins.has("remarkStringify")
              ? "md"
              : "tree"
      if (plugins.has("rehypeKatex") || plugins.has("rehypeMathjax"))
        return countSkip("a math renderer (KaTeX, MathJax), deferred")
      const title = findTestTitle(ancestors, constants) ?? "remark-math"
      const count = (counts.get(title) ?? 0) + 1
      counts.set(title, count)
      found.push({
        kind,
        title: count === 1 ? title : `${title} (${count})`,
        input,
        math: { ...math },
      })
    },
  })
  return found
}

// The plugins of a `unified().use(a).use(b, options)` chain, each with its options node
function readChain(
  node: AnyNode,
  constants: Map<string, AnyNode>,
  depth = 0,
): Map<string, AnyNode | undefined> | undefined {
  if (node.type === "Identifier") {
    const bound = constants.get(node.name)
    return bound && depth < 10 ? readChain(bound, constants, depth + 1) : undefined
  }
  if (node.type !== "CallExpression") return undefined
  if (isCallTo(node, "unified")) return new Map()
  if (
    node.callee.type !== "MemberExpression" ||
    node.callee.property.type !== "Identifier"
  )
    return undefined
  if (node.callee.property.name !== "use") return undefined
  const plugins = readChain(node.callee.object, constants, depth + 1)
  const plugin = node.arguments[0]
  if (!plugins || plugin?.type !== "Identifier") return undefined
  plugins.set(plugin.name, node.arguments[1])
  return plugins
}

function isAssertThrows(callee: AnyNode) {
  return (
    callee.type === "MemberExpression" &&
    callee.property.type === "Identifier" &&
    callee.property.name === "throws"
  )
}

function isCallTo(call: CallExpression, name: string) {
  return call.callee.type === "Identifier" && call.callee.name === name
}

function withOptions(name: string, options: Record<string, unknown>) {
  return Object.keys(options).length > 0 ? [name, options] : [name]
}

function countSkip(reason: string) {
  skipped.set(reason, (skipped.get(reason) ?? 0) + 1)
  return undefined
}
