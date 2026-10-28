import { readdirSync, readFileSync } from "node:fs"
import { basename, join, resolve } from "node:path"
import { parse } from "acorn"
import type { AnyNode } from "acorn"
import { ancestor } from "acorn-walk"
import { writeGroups } from "../actions/group/write.ts"
import { normalizeHtml } from "../actions/html/normalize.ts"
import { isStressInput } from "../actions/input/check.ts"
import { createPipeline, renderMarkdown } from "../actions/markdown/render.ts"
import type { RenderOptions } from "../actions/markdown/render.ts"
import { evaluateNode, isObject } from "../actions/node/evaluate.ts"
import { resolveConstants } from "../actions/node/scope.ts"
import { fetchSource } from "../actions/source/fetch.ts"
import { isAssertEqual } from "../actions/test/assert.ts"
import { findTestTitle } from "../actions/test/find.ts"

const OUTPUT = resolve(import.meta.dirname, "..", "tests", "md")
const ORG = "https://github.com/micromark"

// Each extension's repository and the construct file its tests and fixtures land in
const REPOSITORIES = [
  {
    name: "micromark-extension-gfm",
    commit: "cd4e38f455e5872eefc83c9036ca621ba954cad8",
    construct: "gfm",
  },
  {
    name: "micromark-extension-gfm-table",
    commit: "1511204dae5a01e81588cee417ecd4fb8d2c8aff",
    construct: "gfm-table",
  },
  {
    name: "micromark-extension-gfm-autolink-literal",
    commit: "618170c86639742036ecf666d975a6ebac5aac50",
    construct: "gfm-autolink-literal",
  },
  {
    name: "micromark-extension-gfm-footnote",
    commit: "0a62fad40470f2447707020c52d38d1494199ee1",
    construct: "gfm-footnote",
  },
  {
    name: "micromark-extension-gfm-strikethrough",
    commit: "895451f924c543c6a528e80b7340236e164eb384",
    construct: "gfm-strikethrough",
  },
  {
    name: "micromark-extension-gfm-task-list-item",
    commit: "7902c38c76a6dea80928a54829313b702615d1e1",
    construct: "gfm-task-list-item",
  },
]
// GFM spec categories onto construct files; GitHub's tag filter is in neither micromark nor unified
const SPEC_CONSTRUCTS: Record<string, string | undefined> = {
  Tables: "gfm-table",
  "Task list items": "gfm-task-list-item",
  Strikethrough: "gfm-strikethrough",
  Autolinks: "gfm-autolink-literal",
  "Disallowed Raw HTML": undefined,
}
// Extension arguments onto textlane options: `md.gfm` keeps remark-gfm's names, `md.toHtml`
// remark-rehype's
const ARGUMENTS: Record<string, Record<string, [keyof RenderOptions, string]>> = {
  gfmStrikethrough: { singleTilde: ["gfm", "singleTilde"] },
  gfmFootnoteHtml: {
    clobberPrefix: ["toHtml", "clobberPrefix"],
    label: ["toHtml", "footnoteLabel"],
    labelTagName: ["toHtml", "footnoteLabelTagName"],
    backLabel: ["toHtml", "footnoteBackLabel"],
  },
}

type Group = {
  description: string
  source: { repository: string; commit: string; path: string; license: string }
  pipeline: unknown[]
  tests: object[]
}
type Repository = (typeof REPOSITORIES)[number]

const skipped = new Map<string, number>()
const seen = new Set<string>()
importGfm()

function importGfm() {
  let written = 0
  for (const repository of REPOSITORIES) {
    const root = fetchSource({
      repository: `${ORG}/${repository.name}`,
      commit: repository.commit,
    })
    const byConstruct = new Map<string, Map<string, Group>>()
    const add = (construct: string, group: Omit<Group, "tests">, test: object) => {
      const groups = byConstruct.get(construct) ?? new Map<string, Group>()
      byConstruct.set(construct, groups)
      const existing = groups.get(group.description) ?? { ...group, tests: [] }
      existing.tests.push(test)
      groups.set(group.description, existing)
      written += 1
    }
    if (repository.name === "micromark-extension-gfm") {
      for (const example of readSpec(join(root, "test", "spec.js"))) {
        const construct = SPEC_CONSTRUCTS[example.category]
        if (!construct) {
          countSkip("GitHub's tag filter, in neither micromark nor unified")
          continue
        }
        const options = { allowDangerousHtml: true, gfm: {} }
        const test = createTest({
          title: `example ${example.number}`,
          markdown: example.input,
          html: example.output,
          by: "the GFM spec expects",
          options,
        })
        if (test)
          add(
            construct,
            createGroup({
              repository,
              description: `GFM spec: ${example.category}`,
              path: "test/spec.js",
              options,
              license: "CC-BY-SA-4.0",
            }),
            test,
          )
      }
    }
    for (const found of extractInline(
      repository,
      readFileSync(join(root, "test", "index.js"), "utf8"),
    )) {
      const suffix = describeOptions(found.options)
      const group = createGroup({
        repository,
        description: `${repository.name}${suffix}`,
        path: "test/index.js",
        options: found.options,
      })
      const test = createTest({
        title: found.title,
        markdown: found.markdown,
        html: found.html,
        by: "micromark renders",
        options: found.options,
      })
      if (test) add(repository.construct, group, test)
    }
    const fixtures = join(root, "test", "fixtures")
    for (const name of readdirSync(fixtures)
      .filter(file => file.endsWith(".md"))
      .sort()) {
      const markdown = readFileSync(join(fixtures, name), "utf8")
      const html = readFileSync(join(fixtures, name.replace(/\.md$/, ".html")), "utf8")
      const options = { allowDangerousHtml: true, gfm: {} }
      const test = createTest({
        title: `fixture ${basename(name, ".md")}`,
        markdown,
        html,
        by: "GitHub renders",
        options,
      })
      if (test)
        add(
          repository.construct,
          createGroup({
            repository,
            description: `${repository.name} fixtures`,
            path: "test/fixtures",
            options,
          }),
          test,
        )
    }
    for (const [construct, groups] of byConstruct) {
      const path = join(OUTPUT, `${construct}.json`)
      writeGroups({
        path,
        repository: `${ORG}/${repository.name}`,
        groups: [...groups.values()],
      })
    }
  }
  console.log(`gfm: ${written} cases`)
  for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)
}

function createGroup(props: {
  repository: Repository
  description: string
  path: string
  options: RenderOptions
  license?: string
}) {
  return {
    description: props.description,
    source: {
      repository: `${ORG}/${props.repository.name}`,
      commit: props.repository.commit,
      path: props.path,
      license: props.license ?? "MIT",
    },
    pipeline: createPipeline(props.options),
  }
}

// Unified's output is the expectation; upstream's HTML is kept where it means something else
function createTest(props: {
  title: string
  markdown: string
  html: string
  by: string
  options: RenderOptions
}) {
  if (isStressInput(props.markdown)) return countSkip("a stress test")
  if (!props.markdown.isWellFormed())
    return countSkip("a lone surrogate, which UTF-8 input cannot hold")
  const key = `${JSON.stringify(createPipeline(props.options))}\u0000${props.markdown}`
  if (seen.has(key)) return countSkip("duplicate of an earlier case")
  seen.add(key)
  const actual = renderMarkdown({ ...props.options, markdown: props.markdown })
  const differs = normalizeHtml(actual) !== normalizeHtml(props.html)
  return {
    description: props.title,
    ...(differs ? { comment: `${props.by}: ${props.html}` } : {}),
    input: props.markdown,
    expected: { html: actual },
  }
}

function readSpec(path: string) {
  const program = parse(readFileSync(path, "utf8"), {
    ecmaVersion: "latest",
    sourceType: "module",
  })
  let examples: unknown
  ancestor(program, {
    VariableDeclarator(node) {
      if (node.id.type === "Identifier" && node.id.name === "spec")
        examples = evaluateNode(node.init, new Map())
    },
  })
  if (!Array.isArray(examples)) throw new Error(`no spec examples in ${path}`)
  return examples.flatMap((example, index) =>
    isObject(example) &&
    "category" in example &&
    "input" in example &&
    "output" in example
      ? [
          {
            number: index + 1,
            category: String(example.category),
            input: String(example.input),
            output: String(example.output),
          },
        ]
      : [],
  )
}

function extractInline(repository: Repository, source: string) {
  const program = parse(source, { ecmaVersion: "latest", sourceType: "module" })
  const found: {
    title: string
    markdown: string
    html: string
    options: RenderOptions
  }[] = []
  const counts = new Map<string, number>()
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      if (!isAssertEqual(node.callee)) return
      const [call, expected] = node.arguments
      if (call?.type !== "CallExpression" || call.callee.type !== "Identifier") return
      if (call.callee.name !== "micromark") return
      const constants = resolveConstants({ program, ancestors })
      const markdown = evaluateNode(call.arguments[0], constants)
      const html = evaluateNode(expected, constants)
      if (typeof markdown !== "string" || typeof html !== "string")
        return countSkip("not statically evaluable")
      const options = readOptions(call.arguments[1], constants)
      if (typeof options === "string") return countSkip(options)
      const title = findTestTitle(ancestors, constants) ?? repository.name
      const count = (counts.get(title) ?? 0) + 1
      counts.set(title, count)
      found.push({
        title: count === 1 ? title : `${title} (${count})`,
        markdown,
        html,
        options,
      })
    },
  })
  return found
}

// `{extensions: [gfmX(args?)], htmlExtensions: [gfmXHtml(args?)], allowDangerousHtml}`; a
// string result names why the case is skipped
function readOptions(
  node: AnyNode | undefined,
  constants: Map<string, AnyNode>,
): RenderOptions | string {
  const target = node?.type === "Identifier" ? constants.get(node.name) : node
  if (!target) return "no GFM extension enabled"
  if (target.type !== "ObjectExpression") return "options not statically evaluable"
  const options: Required<RenderOptions> = {
    allowDangerousHtml: false,
    gfm: {},
    toHtml: {},
  }
  let enabled = false
  for (const property of target.properties) {
    if (property.type !== "Property" || property.key.type !== "Identifier")
      return "options not statically evaluable"
    const key = property.key.name
    if (key === "allowDangerousProtocol") continue
    if (key === "allowDangerousHtml") {
      options.allowDangerousHtml = evaluateNode(property.value, constants) === true
      continue
    }
    if (key !== "extensions" && key !== "htmlExtensions")
      return `micromark options without a pipeline equivalent (${key})`
    if (property.value.type !== "ArrayExpression")
      return "options not statically evaluable"
    for (const element of property.value.elements) {
      if (element?.type !== "CallExpression" || element.callee.type !== "Identifier")
        return "options not statically evaluable"
      const name = element.callee.name
      if (!name.startsWith("gfm")) return `a non-GFM extension (${name})`
      enabled = true
      const argument = element.arguments[0]
        ? evaluateNode(element.arguments[0], constants)
        : {}
      if (!isObject(argument)) return "extension options not statically evaluable"
      for (const [option, value] of Object.entries(argument)) {
        const mapped = ARGUMENTS[name]?.[option]
        if (!mapped)
          return `an extension option without a pipeline equivalent (${name} ${option})`
        const [where, renamed] = mapped
        if (where === "gfm") options.gfm[renamed] = value
        else options.toHtml[renamed] = value
      }
    }
  }
  if (!enabled) return "no GFM extension enabled"
  return {
    allowDangerousHtml: options.allowDangerousHtml,
    gfm: options.gfm,
    ...(Object.keys(options.toHtml).length > 0 ? { toHtml: options.toHtml } : {}),
  }
}

function describeOptions(options: RenderOptions) {
  const parts = [
    ...(options.allowDangerousHtml ? ["dangerous HTML allowed"] : []),
    ...Object.entries({ ...options.gfm, ...options.toHtml }).map(
      ([key, value]) => `${key} ${JSON.stringify(value)}`,
    ),
  ]
  return parts.length === 0 ? "" : ` (${parts.join(", ")})`
}

function countSkip(reason: string) {
  skipped.set(reason, (skipped.get(reason) ?? 0) + 1)
  return undefined
}
