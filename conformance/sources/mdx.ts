import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { compile } from "@mdx-js/mdx"
import { parse } from "acorn"
import type { AnyNode, CallExpression, Program } from "acorn"
import { simple } from "acorn-walk"
import remarkDirective from "remark-directive"
import remarkFrontmatter from "remark-frontmatter"
import remarkGfm from "remark-gfm"
import remarkMath from "remark-math"
import { writeGroups } from "../actions/group/write.ts"
import { isStressInput } from "../actions/input/check.ts"
import { parseMarkdown } from "../actions/markdown/render.ts"
import {
  createMdastPipeline,
  describeMdastOptions,
  extractMdastCases,
} from "../actions/mdast/extract.ts"
import type { MdastSyntax } from "../actions/mdast/extract.ts"
import { evaluateNode, isObject } from "../actions/node/evaluate.ts"
import { fetchSource } from "../actions/source/fetch.ts"

const TESTS = resolve(import.meta.dirname, "..", "tests")
const MICROMARK = [
  {
    url: "https://github.com/micromark/micromark-extension-mdx-jsx",
    commit: "ad0a49c716c9b0044b3db48c9e43d405eda183b7",
    path: "test/index.js",
  },
  {
    url: "https://github.com/micromark/micromark-extension-mdx-expression",
    commit: "2891b75ff9e985c6df208a47348e76ced05dbfed",
    path: "test/index.js",
  },
  {
    url: "https://github.com/micromark/micromark-extension-mdxjs-esm",
    commit: "3fdf3d3e597c707ac08ca94ba52d99d88f87ddfe",
    path: "test/index.js",
  },
  {
    url: "https://github.com/micromark/micromark-extension-mdxjs",
    commit: "25cda5802db19d21b3bd4d9e6ea4cc66bd027c78",
    path: "test.js",
  },
]
const MDAST = [
  {
    url: "https://github.com/syntax-tree/mdast-util-mdx-jsx",
    commit: "998d98d0aa29fe9ed12c636ddf1ed39f9a018096",
    path: "test.js",
  },
  {
    url: "https://github.com/syntax-tree/mdast-util-mdx-expression",
    commit: "7c04918301539bd9bdf624bd7271127c3064ca22",
    path: "test.js",
  },
  {
    url: "https://github.com/syntax-tree/mdast-util-mdxjs-esm",
    commit: "8d05c28d15ec5b690e7fbb08d703b0752d431109",
    path: "test.js",
  },
  {
    url: "https://github.com/syntax-tree/mdast-util-mdx",
    commit: "9a8c096cf9cf48ec5be34514c7ceabfcbf0ddfff",
    path: "test.js",
  },
]
const MDX = {
  url: "https://github.com/mdx-js/mdx",
  commit: "52285a6758fa078ec57f3d4bd8803d9cbfb12065",
}
// `md.mdx` keeps remark-mdx's serializer option names
const SYNTAX: MdastSyntax = {
  plugin: "md.mdx",
  extension: "mdx",
  isExtension: name => /^mdx/.test(name),
  options: new Set(["quote", "tightSelfClosing", "printWidth"]),
}
// compile options a case can carry; path-dependent ones (development, baseUrl) and source maps can't
const COMPILE_OPTIONS = new Set([
  "jsx",
  "jsxRuntime",
  "jsxImportSource",
  "pragma",
  "pragmaFrag",
  "pragmaImportSource",
  "providerImportSource",
  "outputFormat",
  "format",
  "elementAttributeNameCase",
  "stylePropertyNameCase",
  "tableCellAlignToStyle",
])
// remark plugins a compile test passes, onto the textlane stage each one is
const REMARK_PLUGINS = {
  remarkGfm: ["md.gfm", remarkGfm],
  remarkMath: ["md.math", remarkMath],
  remarkFrontmatter: ["md.frontmatter", remarkFrontmatter],
  remarkDirective: ["md.directive", remarkDirective],
} as const

type Source = { repository: string; commit: string; path: string; license: string }
type Group = { description: string; source: Source; pipeline: unknown[]; tests: object[] }

const skipped = new Map<string, number>()
let written = 0
importSyntax()
await importCompile()
console.log(`mdx: ${written} cases`)
for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)

// Every literal input the micromark extensions test, as a tree or error case through remark-mdx;
// micromark's own HTML blanks the MDX out and says nothing about it
function importSyntax() {
  const path = join(TESTS, "md", "mdx.json")
  const seen = new Set<string>()
  for (const repository of MICROMARK) {
    const program = readProgram(repository, repository.path)
    const trees: object[] = []
    const errors: object[] = []
    for (const input of findLiteralCalls(program, "micromark")) {
      if (seen.has(input)) continue
      seen.add(input)
      if (isStressInput(input) || !input.isWellFormed()) {
        countSkip("a stress test or a lone surrogate")
        continue
      }
      const outcome = parseMdx(input)
      if ("error" in outcome)
        errors.push({ description: describeInput(input), input, expected: outcome })
      else trees.push({ description: describeInput(input), input, expected: outcome })
      written += 1
    }
    const source = {
      repository: repository.url,
      commit: repository.commit,
      path: repository.path,
      license: "MIT",
    }
    const name = repository.url.split("/").pop()
    writeGroups({
      path,
      repository: repository.url,
      groups: [
        ...(trees.length > 0
          ? [
              {
                description: `${name} (tree)`,
                source,
                pipeline: [["md.parse"], ["md.mdx"]],
                tests: trees,
              },
            ]
          : []),
        ...(errors.length > 0
          ? [
              {
                description: `${name} (errors)`,
                source,
                pipeline: [["md.parse"], ["md.mdx"]],
                tests: errors,
              },
            ]
          : []),
      ],
    })
  }
  for (const repository of MDAST) {
    const root = fetchSource({ repository: repository.url, commit: repository.commit })
    const groups = new Map<string, Group>()
    for (const found of extractMdastCases({
      source: readFileSync(join(root, repository.path), "utf8"),
      syntax: SYNTAX,
      countSkip,
    })) {
      const description = `${repository.url.split("/").pop()} ${found.kind}${describeMdastOptions(found.options)}`
      const group = groups.get(description) ?? {
        description,
        source: {
          repository: repository.url,
          commit: repository.commit,
          path: repository.path,
          license: "MIT",
        },
        pipeline: createMdastPipeline({
          syntax: SYNTAX,
          kind: found.kind,
          options: found.options,
        }),
        tests: [],
      }
      group.tests.push(found.test)
      groups.set(description, group)
      written += 1
    }
    writeGroups({ path, repository: repository.url, groups: [...groups.values()] })
  }
}

// `compile(literal, options?)` in @mdx-js/mdx's tests, compiled by the pinned @mdx-js/mdx: the
// JavaScript is the expectation, exactly; upstream mostly renders it, which only a JS runner could
async function importCompile() {
  const groups = new Map<string, Group>()
  for (const path of ["packages/mdx/test/compile.js", "packages/mdx/test/syntax.js"]) {
    const program = readProgram(MDX, path)
    const calls: CallExpression[] = []
    simple(program, {
      CallExpression(node) {
        if (node.callee.type === "Identifier" && node.callee.name === "compile")
          calls.push(node)
      },
    })
    for (const call of calls) {
      const input = evaluateNode(call.arguments[0], new Map())
      if (typeof input !== "string") {
        countSkip("compile input not statically evaluable")
        continue
      }
      const options = readCompileOptions(call.arguments[1])
      if (typeof options === "string") {
        countSkip(options)
        continue
      }
      const outcome = await compileMdx(input, options)
      const markdown = options.compile.format === "md"
      const compileOptions = Object.fromEntries(
        Object.entries(options.compile).filter(([key]) => key !== "format"),
      )
      const pipeline = [
        ["md.parse"],
        ...(markdown ? [] : [["md.mdx"]]),
        ...options.stages,
        ["md.toHtml"],
        Object.keys(compileOptions).length > 0
          ? ["mdx.compile", compileOptions]
          : ["mdx.compile"],
      ]
      const key = JSON.stringify(pipeline)
      const description = `@mdx-js/mdx compile${key.includes('"mdx.compile",{') ? ` ${JSON.stringify(compileOptions)}` : ""}${markdown ? " (format md)" : ""}${options.stages.length > 0 ? ` with ${options.stages.map(stage => (stage.length > 1 ? `${stage[0]} ${JSON.stringify(stage[1])}` : stage[0])).join(", ")}` : ""}`
      const group = groups.get(key) ?? {
        description,
        source: { repository: MDX.url, commit: MDX.commit, path, license: "MIT" },
        pipeline,
        tests: [],
      }
      if (group.tests.some(test => "input" in test && test.input === input)) continue
      group.tests.push({ description: describeInput(input), input, expected: outcome })
      groups.set(key, group)
      written += 1
    }
  }
  writeGroups({
    path: join(TESTS, "mdx", "compile.json"),
    repository: MDX.url,
    groups: [...groups.values()],
  })
}

function readCompileOptions(node: AnyNode | undefined) {
  if (!node) return { compile: {}, stages: [], plugins: [] }
  if (node.type !== "ObjectExpression") return "compile options not statically evaluable"
  const compileOptions: Record<string, unknown> = {}
  const stages: unknown[][] = []
  const plugins: unknown[] = []
  for (const property of node.properties) {
    if (property.type !== "Property" || property.key.type !== "Identifier")
      return "compile options not statically evaluable"
    const key = property.key.name
    if (key === "remarkPlugins") {
      if (property.value.type !== "ArrayExpression")
        return "remark plugins not statically evaluable"
      for (const element of property.value.elements) {
        const name =
          element?.type === "Identifier"
            ? element.name
            : element?.type === "ArrayExpression" &&
                element.elements[0]?.type === "Identifier"
              ? element.elements[0].name
              : ""
        const options =
          element?.type === "ArrayExpression"
            ? evaluateNode(element.elements[1], new Map())
            : undefined
        const known = Object.entries(REMARK_PLUGINS).find(
          ([identifier]) => identifier === name,
        )?.[1]
        if (!known)
          return `a remark plugin without a textlane stage (${name || "inline"})`
        stages.push(options === undefined ? [known[0]] : [known[0], options])
        plugins.push(options === undefined ? known[1] : [known[1], options])
      }
      continue
    }
    if (!COMPILE_OPTIONS.has(key)) return `a compile option a case cannot carry (${key})`
    const value = evaluateNode(property.value, new Map())
    if (value === undefined) return "compile options not statically evaluable"
    compileOptions[key] = value
  }
  return { compile: compileOptions, stages, plugins }
}

async function compileMdx(
  input: string,
  options: { compile: Record<string, unknown>; plugins: unknown[] },
) {
  try {
    const file = await Reflect.apply(compile, undefined, [
      input,
      { ...options.compile, remarkPlugins: options.plugins },
    ])
    return { js: String(file) }
  } catch (error) {
    return { error: describeError(error) }
  }
}

function parseMdx(input: string) {
  try {
    return { tree: parseMarkdown(input, { mdx: {} }) }
  } catch (error) {
    return { error: describeError(error) }
  }
}

// A VFileMessage's `source:ruleId` is the code; its reason the message a runner matches a part of
function describeError(error: unknown) {
  if (isObject(error) && "ruleId" in error && "source" in error && "reason" in error) {
    return {
      code: `${String(error.source)}:${String(error.ruleId)}`,
      message: String(error.reason),
    }
  }
  return {
    code: "mdx-failed",
    message: error instanceof Error ? error.message : String(error),
  }
}

function findLiteralCalls(program: Program, name: string) {
  const inputs: string[] = []
  simple(program, {
    CallExpression(node) {
      if (node.callee.type !== "Identifier" || node.callee.name !== name) return
      const input = evaluateNode(node.arguments[0], new Map())
      if (typeof input === "string") inputs.push(input)
    },
  })
  return inputs
}

function describeInput(input: string) {
  const line = JSON.stringify(input)
  return line.length > 60 ? `${line.slice(0, 57)}…` : line
}

function readProgram(repository: { url: string; commit: string }, path: string) {
  const root = fetchSource({ repository: repository.url, commit: repository.commit })
  return parse(readFileSync(join(root, path), "utf8"), {
    ecmaVersion: "latest",
    sourceType: "module",
  })
}

function countSkip(reason: string) {
  skipped.set(reason, (skipped.get(reason) ?? 0) + 1)
  return undefined
}
