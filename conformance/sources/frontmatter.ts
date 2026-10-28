import { isDeepStrictEqual } from "node:util"
import { existsSync, readdirSync, readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { parse } from "acorn"
import type { AnyNode, CallExpression, Program } from "acorn"
import { ancestor, simple } from "acorn-walk"
import { writeGroups } from "../actions/group/write.ts"
import { normalizeHtml } from "../actions/html/normalize.ts"
import { isStressInput } from "../actions/input/check.ts"
import {
  createPipeline,
  parseMarkdown,
  renderMarkdown,
  stringifyTree,
} from "../actions/markdown/render.ts"
import type { RenderOptions } from "../actions/markdown/render.ts"
import {
  createMdastPipeline,
  describeMdastOptions,
  extractMdastCases,
  isFrontmatterOptions,
} from "../actions/mdast/extract.ts"
import type { MdastSyntax } from "../actions/mdast/extract.ts"
import { evaluateNode } from "../actions/node/evaluate.ts"
import { resolveConstants } from "../actions/node/scope.ts"
import { fetchSource } from "../actions/source/fetch.ts"
import { isAssertEqual } from "../actions/test/assert.ts"
import { findTestTitle } from "../actions/test/find.ts"

const OUTPUT = join(resolve(import.meta.dirname, "..", "tests", "md"), "frontmatter.json")
const MICROMARK = {
  url: "https://github.com/micromark/micromark-extension-frontmatter",
  commit: "f05bf24461d31041f37f4562fd48877af4dcc67b",
}
const MDAST = {
  url: "https://github.com/syntax-tree/mdast-util-frontmatter",
  commit: "d76616b88bdafdbbcd97247c4d3a8a41cc71ae48",
}
const REMARK = {
  url: "https://github.com/remarkjs/remark-frontmatter",
  commit: "1125009e530386bc8c2ba497087f7f2a3ae99ddf",
}

// `md.frontmatter` takes remark-frontmatter's options whole: a preset, a matter, or a list
const FRONTMATTER: MdastSyntax = {
  plugin: "md.frontmatter",
  extension: "frontmatter",
  isExtension: name => name.startsWith("frontmatter"),
  options: "positional",
}

type Source = { repository: string; commit: string; path: string; license: string }
type Group = { description: string; source: Source; pipeline: unknown[]; tests: object[] }
type Repository = { url: string; commit: string }

const skipped = new Map<string, number>()
const seen = new Set<string>()
const groups = new Map<string, Group>()
let written = 0
importFrontmatter()

function importFrontmatter() {
  const micromark = parseFile(MICROMARK, "test/index.js")
  for (const found of extractHtmlCases(micromark)) {
    const options: RenderOptions = {
      allowDangerousHtml: found.allowDangerousHtml,
      frontmatter: found.frontmatter,
    }
    const actual = renderMarkdown({ ...options, markdown: found.input })
    const differs = normalizeHtml(actual) !== normalizeHtml(found.html)
    const suffix =
      found.frontmatter === null ? "" : ` ${JSON.stringify(found.frontmatter)}`
    add({
      description: `micromark-extension-frontmatter${suffix}${found.allowDangerousHtml ? " with dangerous HTML allowed" : ""}`,
      source: source(MICROMARK, "test/index.js"),
      pipeline: createPipeline(options),
      test: {
        description: found.title,
        ...(differs ? { comment: `micromark renders: ${found.html}` } : {}),
        input: found.input,
        expected: { html: actual },
      },
    })
  }
  for (const [repository, path] of [
    [MICROMARK, "test/index.js"],
    [REMARK, "test/index.js"],
  ] as const) {
    for (const found of extractErrorCases(parseFile(repository, path))) {
      add({
        // Each throw has its own options, so its own pipeline and group
        description: `${repository.url.split("/").pop()} invalid options ${JSON.stringify(found.options)}`,
        source: source(repository, path),
        pipeline: [["md.parse"], ["md.frontmatter", found.options]],
        test: {
          description: found.title,
          input: "",
          expected: { error: { code: "invalid-options", message: found.message } },
        },
      })
    }
  }
  const mdastSource = readFileSync(
    join(fetchSource({ repository: MDAST.url, commit: MDAST.commit }), "test.js"),
    "utf8",
  )
  for (const found of extractMdastCases({
    source: mdastSource,
    syntax: FRONTMATTER,
    countSkip,
  })) {
    add({
      description: `mdast-util-frontmatter ${found.kind}${describeMdastOptions(found.options)}`,
      source: source(MDAST, "test.js"),
      pipeline: createMdastPipeline({
        syntax: FRONTMATTER,
        kind: found.kind,
        options: found.options,
      }),
      test: found.test,
    })
  }
  importRemarkFixtures()
  for (const repository of [MICROMARK, MDAST, REMARK]) {
    const own = [...groups.values()].filter(
      group => group.source.repository === repository.url,
    )
    writeGroups({ path: OUTPUT, repository: repository.url, groups: own })
  }
  console.log(`frontmatter: ${written} cases`)
  for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)
}

// A fixture holds input.md, tree.json, an optional output.md and options in config.json
function importRemarkFixtures() {
  const root = join(
    fetchSource({ repository: REMARK.url, commit: REMARK.commit }),
    "test",
    "fixtures",
  )
  for (const name of readdirSync(root).sort()) {
    const configPath = join(root, name, "config.json")
    const config: unknown = existsSync(configPath)
      ? JSON.parse(readFileSync(configPath, "utf8"))
      : undefined
    const frontmatter =
      config === undefined ? null : isFrontmatterOptions(config) ? config : null
    const input = readFileSync(join(root, name, "input.md"), "utf8")
    const tree = parseMarkdown(input, { frontmatter })
    const md = stringifyTree({ tree, frontmatter })
    if (
      !isDeepStrictEqual(
        tree,
        JSON.parse(readFileSync(join(root, name, "tree.json"), "utf8")),
      )
    ) {
      console.log(`  remark-frontmatter fixture ${name} tree drifts from its snapshot`)
    }
    // No output.md means the output is the input, as remark-frontmatter's own test reads it
    const outputPath = join(root, name, "output.md")
    const snapshot = existsSync(outputPath) ? readFileSync(outputPath, "utf8") : input
    const drift =
      md === snapshot ? {} : { comment: `remark-frontmatter's snapshot: ${snapshot}` }
    const plugin =
      frontmatter === null ? ["md.frontmatter"] : ["md.frontmatter", frontmatter]
    const suffix = frontmatter === null ? "" : ` ${JSON.stringify(frontmatter)}`
    add({
      description: `remark-frontmatter fixtures (tree)${suffix}`,
      source: source(REMARK, "test/fixtures"),
      pipeline: [["md.parse"], plugin],
      test: { description: `fixture ${name}`, input, expected: { tree } },
    })
    add({
      description: `remark-frontmatter fixtures (output)${suffix}`,
      source: source(REMARK, "test/fixtures"),
      pipeline: [["md.parse"], plugin, ["md.stringify"]],
      test: { description: `fixture ${name}`, ...drift, input, expected: { md } },
    })
  }
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

// `micromark(input, {extensions: [frontmatter(options?)], htmlExtensions: […], allowDangerousHtml?})`
function extractHtmlCases(program: Program) {
  const found: {
    title: string
    input: string
    html: string
    frontmatter: Exclude<RenderOptions["frontmatter"], undefined>
    allowDangerousHtml: boolean
  }[] = []
  const counts = new Map<string, number>()
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      if (!isAssertEqual(node.callee)) return
      const [call, expected] = node.arguments
      if (call?.type !== "CallExpression" || !isCallTo(call, "micromark")) return
      const constants = resolveConstants({ program, ancestors })
      const input = evaluateNode(call.arguments[0], constants)
      const html = evaluateNode(expected, constants)
      if (typeof input !== "string" || typeof html !== "string")
        return countSkip("not statically evaluable")
      if (isStressInput(input)) return countSkip("a stress test")
      const options =
        call.arguments[1]?.type === "Identifier"
          ? constants.get(call.arguments[1].name)
          : call.arguments[1]
      if (options?.type !== "ObjectExpression")
        return countSkip("no frontmatter extension enabled")
      let frontmatter: unknown
      let enabled = false
      let allowDangerousHtml = false
      for (const property of options.properties) {
        if (property.type !== "Property" || property.key.type !== "Identifier") continue
        if (property.key.name === "allowDangerousHtml")
          allowDangerousHtml = evaluateNode(property.value, constants) === true
        if (
          property.key.name !== "extensions" ||
          property.value.type !== "ArrayExpression"
        )
          continue
        for (const element of property.value.elements) {
          if (element?.type !== "CallExpression" || !isCallTo(element, "frontmatter"))
            continue
          enabled = true
          frontmatter = element.arguments[0]
            ? evaluateNode(element.arguments[0], constants)
            : undefined
        }
      }
      if (!enabled) return countSkip("no frontmatter extension enabled")
      if (frontmatter !== undefined && !isFrontmatterOptions(frontmatter))
        return countSkip("options not statically evaluable")
      const title =
        findTestTitle(ancestors, constants) ?? "micromark-extension-frontmatter"
      const count = (counts.get(title) ?? 0) + 1
      counts.set(title, count)
      found.push({
        title: count === 1 ? title : `${title} (${count})`,
        input,
        html,
        frontmatter: frontmatter ?? null,
        allowDangerousHtml,
      })
    },
  })
  return found
}

// `assert.throws(() => frontmatter(options) | remark().use(remarkFrontmatter, options), /^Error: …/)`:
// one stable code for every invalid option, upstream's message as the substring to match
function extractErrorCases(program: Program) {
  const found: { title: string; options: unknown; message: string }[] = []
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      if (
        node.callee.type !== "MemberExpression" ||
        node.callee.property.type !== "Identifier"
      )
        return
      if (node.callee.property.name !== "throws") return
      const [body, pattern] = node.arguments
      if (!body || pattern?.type !== "Literal" || !("regex" in pattern) || !pattern.regex)
        return
      const constants = resolveConstants({ program, ancestors })
      const call = findOptionsCall(body)
      if (!call) return countSkip("a throw outside the options")
      const options = evaluateNode(call, constants)
      if (options === undefined) return countSkip("options not statically evaluable")
      const message = pattern.regex.pattern
        .replace(/^\^/, "")
        .replace(/^Error: /, "")
        .replace(/\\(.)/g, "$1")
      found.push({
        title: findTestTitle(ancestors, constants) ?? "invalid options",
        options,
        message,
      })
    },
  })
  return found
}

function findOptionsCall(node: AnyNode) {
  let options: AnyNode | undefined
  simple(node, {
    CallExpression(call) {
      if (options) return
      if (isCallTo(call, "frontmatter")) options = call.arguments[0]
      const use =
        call.callee.type === "MemberExpression" &&
        call.callee.property.type === "Identifier" &&
        call.callee.property.name === "use"
      if (
        use &&
        call.arguments[0]?.type === "Identifier" &&
        call.arguments[0].name === "remarkFrontmatter"
      )
        options = call.arguments[1]
    },
  })
  return options
}

function parseFile(repository: Repository, path: string) {
  const root = fetchSource({ repository: repository.url, commit: repository.commit })
  return parse(readFileSync(join(root, path), "utf8"), {
    ecmaVersion: "latest",
    sourceType: "module",
  })
}

function source(repository: Repository, path: string): Source {
  return { repository: repository.url, commit: repository.commit, path, license: "MIT" }
}

function isCallTo(call: CallExpression, name: string) {
  return call.callee.type === "Identifier" && call.callee.name === name
}

function countSkip(reason: string) {
  skipped.set(reason, (skipped.get(reason) ?? 0) + 1)
  return undefined
}
