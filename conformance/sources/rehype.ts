import { isDeepStrictEqual } from "node:util"
import { existsSync, readdirSync, readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { parse } from "acorn"
import { ancestor } from "acorn-walk"
import { writeGroups } from "../actions/group/write.ts"
import { collectParseErrors, parseHtml, processHtml } from "../actions/markdown/render.ts"
import { evaluateNode, isObject } from "../actions/node/evaluate.ts"
import { resolveConstants } from "../actions/node/scope.ts"
import { fetchSource } from "../actions/source/fetch.ts"
import { isAssertEqual } from "../actions/test/assert.ts"
import { findTestTitle } from "../actions/test/find.ts"

const OUTPUT = resolve(import.meta.dirname, "..", "tests", "html")
const REHYPE = {
  url: "https://github.com/rehypejs/rehype",
  commit: "cae74c7f729a55c6f407bdb95aaa362b44e7c2c2",
}
const FROM_HTML = {
  url: "https://github.com/syntax-tree/hast-util-from-html",
  commit: "1ade8935dea8d293196f4cfc0463d3e91eb8a49e",
}
// rehype's fixture config feeds both sides; these keys belong to the parser, `reprocess` to the harness
const PARSE_OPTIONS = new Set(["fragment", "verbose", "space"])

type Source = { repository: string; commit: string; path: string; license: string }
type Group = { description: string; source: Source; pipeline: unknown[]; tests: object[] }

const skipped = new Map<string, number>()
let written = 0
importRehype()

function importRehype() {
  const rehype = fetchSource({ repository: REHYPE.url, commit: REHYPE.commit })
  const parseGroups = new Map<string, Group>()
  const add = (groups: Map<string, Group>, group: Omit<Group, "tests">, test: object) => {
    const existing = groups.get(group.description) ?? { ...group, tests: [] }
    existing.tests.push(test)
    groups.set(group.description, existing)
    written += 1
  }
  const fixtures = join(rehype, "test", "fixtures")
  for (const name of readdirSync(fixtures).sort()) {
    const configPath = join(fixtures, name, "config.json")
    const config: Record<string, unknown> = existsSync(configPath)
      ? JSON.parse(readFileSync(configPath, "utf8"))
      : {}
    const parseOptions = Object.fromEntries(
      Object.entries(config).filter(([key]) => PARSE_OPTIONS.has(key)),
    )
    const stringifyOptions = Object.fromEntries(
      Object.entries(config).filter(
        ([key]) => !PARSE_OPTIONS.has(key) && key !== "reprocess",
      ),
    )
    const input = readFileSync(join(fixtures, name, "index.html"), "utf8")
    const tree = parseHtml(input, parseOptions)
    if (
      !isDeepStrictEqual(
        tree,
        JSON.parse(readFileSync(join(fixtures, name, "index.json"), "utf8")),
      )
    ) {
      console.log(`  rehype fixture ${name} tree drifts from its snapshot`)
    }
    const html = processHtml({
      html: input,
      parse: parseOptions,
      stringify: stringifyOptions,
    })
    const snapshot = readFileSync(join(fixtures, name, "result.html"), "utf8")
    const drift = html === snapshot ? {} : { comment: `rehype's snapshot: ${snapshot}` }
    const parse = withOptions("html.parse", parseOptions)
    const source = {
      repository: REHYPE.url,
      commit: REHYPE.commit,
      path: "test/fixtures",
      license: "MIT",
    }
    const options = { ...parseOptions, ...stringifyOptions }
    const suffix = Object.keys(options).length > 0 ? ` ${JSON.stringify(options)}` : ""
    add(
      parseGroups,
      { description: `rehype fixtures (tree)${suffix}`, source, pipeline: [parse] },
      { description: `fixture ${name}`, input, expected: { tree } },
    )
    add(
      parseGroups,
      {
        description: `rehype fixtures (output)${suffix}`,
        source,
        pipeline: [parse, withOptions("html.stringify", stringifyOptions)],
      },
      { description: `fixture ${name}`, ...drift, input, expected: { html } },
    )
  }
  for (const found of extractFromHtml()) {
    const source = {
      repository: FROM_HTML.url,
      commit: FROM_HTML.commit,
      path: "test/index.js",
      license: "MIT",
    }
    const suffix =
      Object.keys(found.options).length > 0 ? ` ${JSON.stringify(found.options)}` : ""
    add(
      parseGroups,
      {
        description: `hast-util-from-html${suffix}`,
        source,
        pipeline: [withOptions("html.parse", found.options)],
      },
      {
        description: found.title,
        input: found.input,
        expected: { tree: parseHtml(found.input, found.options) },
      },
    )
  }
  writeGroups({
    path: join(OUTPUT, "parse.json"),
    repository: REHYPE.url,
    groups: [...parseGroups.values()].filter(
      group => group.source.repository === REHYPE.url,
    ),
  })
  writeGroups({
    path: join(OUTPUT, "parse.json"),
    repository: FROM_HTML.url,
    groups: [...parseGroups.values()].filter(
      group => group.source.repository === FROM_HTML.url,
    ),
  })
  // Parse errors: messages compared on ruleId, source and place, never their wording
  const errors = join(rehype, "test", "parse-error")
  const tests: object[] = []
  for (const name of readdirSync(errors).sort()) {
    const input = readFileSync(join(errors, name, "index.html"), "utf8")
    if (!input.isWellFormed()) {
      countSkip("a lone surrogate, which UTF-8 input cannot hold")
      continue
    }
    const messages = collectParseErrors(input)
    const snapshot: { ruleId?: string; source?: string; place?: unknown }[] = JSON.parse(
      readFileSync(join(errors, name, "messages.json"), "utf8"),
    )
    const expected = snapshot.map(message => ({
      ruleId: message.ruleId ?? "",
      source: message.source ?? "",
      place: message.place ?? null,
    }))
    if (!isDeepStrictEqual(messages, expected))
      console.log(`  rehype parse error ${name} drifts from its snapshot`)
    tests.push({ description: name, input, expected: { messages } })
    written += 1
  }
  writeGroups({
    path: join(OUTPUT, "parse-error.json"),
    repository: REHYPE.url,
    groups: [
      {
        description: "rehype parse errors",
        source: {
          repository: REHYPE.url,
          commit: REHYPE.commit,
          path: "test/parse-error",
          license: "MIT",
        },
        pipeline: [["html.parse", { emitParseErrors: true }]],
        tests,
      },
    ],
  })
  console.log(`rehype: ${written} cases`)
  for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)
}

// `assert.deepEqual(fromHtml(html, options?), …)`; `onerror` is a function and is skipped
function extractFromHtml() {
  const root = fetchSource({ repository: FROM_HTML.url, commit: FROM_HTML.commit })
  const program = parse(readFileSync(join(root, "test", "index.js"), "utf8"), {
    ecmaVersion: "latest",
    sourceType: "module",
  })
  const found: { title: string; input: string; options: Record<string, unknown> }[] = []
  const counts = new Map<string, number>()
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      if (!isAssertEqual(node.callee)) return
      const call = node.arguments[0]
      if (
        call?.type !== "CallExpression" ||
        call.callee.type !== "Identifier" ||
        call.callee.name !== "fromHtml"
      )
        return
      const constants = resolveConstants({ program, ancestors })
      const input = evaluateNode(call.arguments[0], constants)
      const options = call.arguments[1] ? evaluateNode(call.arguments[1], constants) : {}
      if (typeof input !== "string") return countSkip("not statically evaluable")
      if (!isObject(options))
        return countSkip("options not statically evaluable (onerror)")
      if (!input.isWellFormed())
        return countSkip("a lone surrogate, which UTF-8 input cannot hold")
      const title = findTestTitle(ancestors, constants) ?? "hast-util-from-html"
      const count = (counts.get(title) ?? 0) + 1
      counts.set(title, count)
      found.push({
        title: count === 1 ? title : `${title} (${count})`,
        input,
        options: { ...options },
      })
    },
  })
  return found
}

function withOptions(name: string, options: Record<string, unknown>) {
  return Object.keys(options).length > 0 ? [name, options] : [name]
}

function countSkip(reason: string) {
  skipped.set(reason, (skipped.get(reason) ?? 0) + 1)
  return undefined
}
