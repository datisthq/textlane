import { isDeepStrictEqual } from "node:util"
import { existsSync, readdirSync, readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { parse } from "acorn"
import { ancestor } from "acorn-walk"
import { writeGroups } from "../actions/group/write.ts"
import { isStressInput } from "../actions/input/check.ts"
import { parseMarkdown, stringifyTree } from "../actions/markdown/render.ts"
import { createMdastPipeline, extractMdastCases } from "../actions/mdast/extract.ts"
import type { MdastSyntax } from "../actions/mdast/extract.ts"
import { evaluateNode } from "../actions/node/evaluate.ts"
import { resolveConstants } from "../actions/node/scope.ts"
import { fetchSource } from "../actions/source/fetch.ts"
import { isAssertEqual } from "../actions/test/assert.ts"
import { findTestTitle } from "../actions/test/find.ts"

const OUTPUT = join(resolve(import.meta.dirname, "..", "tests", "md"), "directive.json")
const MICROMARK = {
  url: "https://github.com/micromark/micromark-extension-directive",
  commit: "75da8c52a3f40de6485ac1928fdcdefd7ea0c3fb",
}
const MDAST = {
  url: "https://github.com/syntax-tree/mdast-util-directive",
  commit: "cbeb1ead102310647aefd7d3e67aea146f9b984f",
}
const REMARK = {
  url: "https://github.com/remarkjs/remark-directive",
  commit: "23b8f416da165b6ddaa4c5e7e82addaf6dcb96a9",
}

// `md.directive` options keep mdast-util-directive's serializer option names
const DIRECTIVE: MdastSyntax = {
  plugin: "md.directive",
  extension: "directive",
  isExtension: name => name.startsWith("directive"),
  options: new Set([
    "collapseEmptyAttributes",
    "preferShortcut",
    "preferUnquoted",
    "quote",
    "quoteSmart",
  ]),
}

type Source = { repository: string; commit: string; path: string; license: string }
type Group = { description: string; source: Source; pipeline: unknown[]; tests: object[] }

const skipped = new Map<string, number>()
const seen = new Set<string>()
let written = 0
importDirective()

function importDirective() {
  const groups = new Map<string, Group>()
  const add = (props: {
    description: string
    source: Source
    pipeline: unknown[]
    test: object
  }) => {
    const key = `${JSON.stringify(props.pipeline)}\u0000${JSON.stringify(props.test)}`
    if ("input" in props.test && seen.has(key))
      return countSkip("duplicate of an earlier case")
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
  for (const [title, input] of extractMicromarkInputs()) {
    add({
      description: "micromark-extension-directive (tree)",
      source: {
        repository: MICROMARK.url,
        commit: MICROMARK.commit,
        path: "test/index.js",
        license: "MIT",
      },
      pipeline: [["md.parse"], ["md.directive"]],
      test: {
        description: title,
        input,
        expected: { tree: parseMarkdown(input, { directive: {} }) },
      },
    })
  }
  const mdastSource = readFileSync(
    join(fetchSource({ repository: MDAST.url, commit: MDAST.commit }), "test.js"),
    "utf8",
  )
  for (const found of extractMdastCases({
    source: mdastSource,
    syntax: DIRECTIVE,
    countSkip,
  })) {
    const optionsKey = JSON.stringify({
      ...found.options.plugin,
      ...found.options.stringify,
    })
    add({
      description: `mdast-util-directive ${found.kind}${optionsKey === "{}" ? "" : ` ${optionsKey}`}`,
      source: {
        repository: MDAST.url,
        commit: MDAST.commit,
        path: "test.js",
        license: "MIT",
      },
      pipeline: createMdastPipeline({
        syntax: DIRECTIVE,
        kind: found.kind,
        options: found.options,
      }),
      test: found.test,
    })
  }
  const fixtures = join(
    fetchSource({ repository: REMARK.url, commit: REMARK.commit }),
    "test",
    "fixtures",
  )
  for (const name of readdirSync(fixtures).sort()) {
    const input = readFileSync(join(fixtures, name, "input.md"), "utf8")
    const tree = parseMarkdown(input, { directive: {} })
    const md = stringifyTree({ tree, directive: {} })
    if (
      !isDeepStrictEqual(
        tree,
        JSON.parse(readFileSync(join(fixtures, name, "tree.json"), "utf8")),
      )
    ) {
      console.log(`  remark-directive fixture ${name} tree drifts from its snapshot`)
    }
    // No output.md means the output is the input, as remark-directive's own test reads it
    const outputPath = join(fixtures, name, "output.md")
    const snapshot = existsSync(outputPath) ? readFileSync(outputPath, "utf8") : input
    const drift =
      md === snapshot ? {} : { comment: `remark-directive's snapshot: ${snapshot}` }
    const source = {
      repository: REMARK.url,
      commit: REMARK.commit,
      path: "test/fixtures",
      license: "MIT",
    }
    add({
      description: "remark-directive fixtures (tree)",
      source,
      pipeline: [["md.parse"], ["md.directive"]],
      test: { description: `fixture ${name}`, input, expected: { tree } },
    })
    add({
      description: "remark-directive fixtures (output)",
      source,
      pipeline: [["md.parse"], ["md.directive"], ["md.stringify"]],
      test: { description: `fixture ${name}`, ...drift, input, expected: { md } },
    })
  }
  writeGroups({
    path: OUTPUT,
    repository: MICROMARK.url,
    groups: [...groups.values()].filter(group =>
      group.description.startsWith("micromark"),
    ),
  })
  writeGroups({
    path: OUTPUT,
    repository: MDAST.url,
    groups: [...groups.values()].filter(group => group.description.startsWith("mdast")),
  })
  writeGroups({
    path: OUTPUT,
    repository: REMARK.url,
    groups: [...groups.values()].filter(group => group.description.startsWith("remark")),
  })
  console.log(`directive: ${written} cases`)
  for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)
}

// Directives vanish from micromark's HTML unless a handler renders them, so only the inputs
// carry over, as tree cases: the mdast is what livemark's plugins consume
function extractMicromarkInputs() {
  const root = fetchSource({ repository: MICROMARK.url, commit: MICROMARK.commit })
  const program = parse(readFileSync(join(root, "test", "index.js"), "utf8"), {
    ecmaVersion: "latest",
    sourceType: "module",
  })
  const inputs = new Map<string, string>()
  const counts = new Map<string, number>()
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      if (!isAssertEqual(node.callee)) return
      const call = node.arguments[0]
      if (
        call?.type !== "CallExpression" ||
        call.callee.type !== "Identifier" ||
        call.callee.name !== "micromark"
      )
        return
      const constants = resolveConstants({ program, ancestors })
      const input = evaluateNode(call.arguments[0], constants)
      if (typeof input !== "string") return countSkip("not statically evaluable")
      if (isStressInput(input)) return countSkip("a stress test")
      if (!input.isWellFormed())
        return countSkip("a lone surrogate, which UTF-8 input cannot hold")
      if ([...inputs.values()].includes(input))
        return countSkip("duplicate of an earlier case")
      const title = findTestTitle(ancestors, constants) ?? "micromark-extension-directive"
      const count = (counts.get(title) ?? 0) + 1
      counts.set(title, count)
      inputs.set(count === 1 ? title : `${title} (${count})`, input)
    },
  })
  return inputs
}

function countSkip(reason: string) {
  skipped.set(reason, (skipped.get(reason) ?? 0) + 1)
  return undefined
}
