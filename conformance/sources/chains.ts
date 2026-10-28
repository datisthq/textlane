import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { parse } from "acorn"
import type { AnyNode, Program } from "acorn"
import { ancestor } from "acorn-walk"
import rehypeParse from "rehype-parse"
import rehypeRemark from "rehype-remark"
import rehypeStringify from "rehype-stringify"
import remarkMdx from "remark-mdx"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import remarkStringify from "remark-stringify"
import retextEnglish from "retext-english"
import retextLatin from "retext-latin"
import retextStringify from "retext-stringify"
import { unified } from "unified"
import type { PluggableList, Processor } from "unified"
import { writeGroups } from "../actions/group/write.ts"
import { evaluateNode, isObject } from "../actions/node/evaluate.ts"
import { resolveConstants } from "../actions/node/scope.ts"
import { fetchSource } from "../actions/source/fetch.ts"
import { findTestTitle } from "../actions/test/find.ts"

const TESTS = resolve(import.meta.dirname, "..", "tests")

// A plugin a chain uses: its textlane stage and the pinned plugin the reference replays
const PLUGINS: Record<
  string,
  { stage: string; plugin: PluggableList[number]; options?: object }
> = {
  remarkParse: { stage: "md.parse", plugin: remarkParse },
  remarkMdx: { stage: "md.mdx", plugin: remarkMdx },
  remarkStringify: { stage: "md.stringify", plugin: remarkStringify },
  remarkRehype: { stage: "md.toHtml", plugin: remarkRehype },
  rehypeParse: { stage: "html.parse", plugin: rehypeParse },
  rehypeRemark: { stage: "html.toMarkdown", plugin: rehypeRemark },
  rehypeStringify: { stage: "html.stringify", plugin: rehypeStringify },
  retextLatin: { stage: "prose.parse", plugin: retextLatin },
  retextEnglish: {
    stage: "prose.parse",
    plugin: retextEnglish,
    options: { language: "english" },
  },
  retextStringify: { stage: "prose.stringify", plugin: retextStringify },
}
// Factories that start a chain with plugins already used, as their packages do
const FACTORIES: Record<string, string[]> = {
  unified: [],
  remark: ["remarkParse", "remarkStringify"],
  rehype: ["rehypeParse", "rehypeStringify"],
  retext: ["retextLatin", "retextStringify"],
}
const SOURCES = [
  {
    url: "https://github.com/mdx-js/mdx",
    commit: "52285a6758fa078ec57f3d4bd8803d9cbfb12065",
    path: "packages/remark-mdx/test/index.js",
    output: "md/mdx.json",
    name: "remark-mdx",
  },
  {
    url: "https://github.com/retextjs/retext",
    commit: "04580e54d857f86405c1b109f5205be58f1285f8",
    path: "test.js",
    output: "prose/retext.json",
    name: "retext",
  },
  {
    url: "https://github.com/rehypejs/rehype-remark",
    commit: "f740c02d583318904be6cd4b3539e7350983ece8",
    path: "test.js",
    output: "html/to-markdown.json",
    name: "rehype-remark",
  },
  {
    url: "https://github.com/remarkjs/remark-rehype",
    commit: "b5a2e5bb0f14607b6fc79ccbfdf0d518a4f304de",
    path: "test.js",
    output: "md/to-html.json",
    name: "remark-rehype",
  },
]

type Step = { identifier: string; options: Record<string, unknown> }

const skipped = new Map<string, number>()
let written = 0
for (const source of SOURCES) importSource(source)
console.log(`chains: ${written} cases`)
for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)

function importSource(source: (typeof SOURCES)[number]) {
  const root = fetchSource({ repository: source.url, commit: source.commit })
  const program = parse(readFileSync(join(root, source.path), "utf8"), {
    ecmaVersion: "latest",
    sourceType: "module",
  })
  const groups = new Map<string, { pipeline: unknown[]; tests: object[] }>()
  const seen = new Set<string>()
  for (const found of extractRuns(program)) {
    const pipeline = found.steps.map(step => {
      const known = PLUGINS[step.identifier]
      const options = { ...known?.options, ...step.options }
      return Object.keys(options).length > 0 ? [known?.stage, options] : [known?.stage]
    })
    const key = JSON.stringify([pipeline, found.value])
    if (seen.has(key)) continue
    seen.add(key)
    const expected = runReference(found)
    if (!expected) continue
    const groupKey = JSON.stringify(pipeline)
    const group = groups.get(groupKey) ?? { pipeline, tests: [] }
    group.tests.push({
      description: found.title,
      ...(typeof found.value === "string"
        ? { input: found.value }
        : { tree: found.value }),
      expected,
    })
    groups.set(groupKey, group)
    written += 1
  }
  writeGroups({
    path: join(TESTS, source.output),
    repository: source.url,
    groups: [...groups.values()].map((group, index) => ({
      description: `${source.name}${index === 0 ? "" : ` (${index + 1})`}`,
      source: {
        repository: source.url,
        commit: source.commit,
        path: source.path,
        license: "MIT",
      },
      ...group,
    })),
  })
}

// `<chain>.parse(text)`, `.process(text)` / `.processSync(text)` and `.stringify(tree)`
function extractRuns(program: Program) {
  const runs: { title: string; method: string; steps: Step[]; value: string | object }[] =
    []
  const counts = new Map<string, number>()
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      if (
        node.callee.type !== "MemberExpression" ||
        node.callee.property.type !== "Identifier"
      )
        return
      const method = node.callee.property.name
      if (!["parse", "process", "processSync", "stringify"].includes(method)) return
      const constants = resolveConstants({ program, ancestors })
      const steps = readChain(node.callee.object, constants)
      if (!steps) return
      if (steps.some(step => !PLUGINS[step.identifier]))
        return countSkip("a chain with a plugin outside the corpus")
      const value = evaluateNode(node.arguments[0], constants)
      if (typeof value !== "string" && !isObject(value))
        return countSkip("input not statically evaluable")
      const title = findTestTitle(ancestors, constants) ?? method
      const count = (counts.get(title) ?? 0) + 1
      counts.set(title, count)
      runs.push({
        title: count === 1 ? title : `${title} (${count})`,
        method,
        steps,
        value,
      })
    },
  })
  return runs
}

// The steps of a chain, each plugin with its statically evaluated options; undefined when the
// chain is not one, or passes options a case cannot hold (functions, processors)
function readChain(
  node: AnyNode,
  constants: Map<string, AnyNode>,
  depth = 0,
): Step[] | undefined {
  if (depth > 10) return undefined
  if (node.type === "Identifier") {
    const bound = constants.get(node.name)
    return bound ? readChain(bound, constants, depth + 1) : undefined
  }
  if (node.type !== "CallExpression") return undefined
  if (node.callee.type === "Identifier") {
    const factory = FACTORIES[node.callee.name]
    return factory ? factory.map(identifier => ({ identifier, options: {} })) : undefined
  }
  if (
    node.callee.type !== "MemberExpression" ||
    node.callee.property.type !== "Identifier"
  )
    return undefined
  const steps = readChain(node.callee.object, constants, depth + 1)
  if (!steps) return undefined
  const method = node.callee.property.name
  if (method === "freeze" || method === "data") return steps
  if (method !== "use") return undefined
  const plugin = node.arguments[0]
  if (plugin?.type !== "Identifier") return undefined
  const options = node.arguments[1] ? evaluateNode(node.arguments[1], constants) : {}
  if (!isObject(options)) return undefined
  // A stringifier used after a factory's own replaces it, as unified keeps one compiler
  const rest = steps.filter(step => step.identifier !== plugin.name)
  return [...rest, { identifier: plugin.name, options: { ...options } }]
}

function runReference(found: { method: string; steps: Step[]; value: string | object }) {
  // The plugins go to unified untyped: their option types differ per plugin
  let processor: Processor = unified()
  for (const step of found.steps) {
    processor = Reflect.apply(processor.use, processor, [
      PLUGINS[step.identifier]?.plugin,
      step.options,
    ])
  }
  const last = found.steps.at(-1)?.identifier ?? ""
  try {
    if (found.method === "parse" && typeof found.value === "string") {
      return {
        tree: JSON.parse(
          JSON.stringify(processor.parse(found.value), (key, value) =>
            key === "estree" ? undefined : value,
          ),
        ),
      }
    }
    if (found.method === "stringify" && isObject(found.value)) {
      return { md: String(Reflect.apply(processor.stringify, processor, [found.value])) }
    }
    if (typeof found.value === "string") {
      const output = String(processor.processSync(found.value))
      if (last === "rehypeStringify") return { html: output }
      if (last === "retextStringify") return { txt: output }
      return { md: output }
    }
  } catch {
    return countSkip("the reference rejects the run")
  }
  return countSkip("a run the reference cannot replay")
}

function countSkip(reason: string) {
  skipped.set(reason, (skipped.get(reason) ?? 0) + 1)
  return undefined
}
