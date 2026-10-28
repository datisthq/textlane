import { readdirSync, readFileSync } from "node:fs"
import { basename, join, resolve } from "node:path"
import { parse } from "acorn"
import { ancestor } from "acorn-walk"
import { toHtml } from "hast-util-to-html"
import { h, s } from "hastscript"
import { writeGroups } from "../actions/group/write.ts"
import { evaluateNode, isObject } from "../actions/node/evaluate.ts"
import { resolveConstants } from "../actions/node/scope.ts"
import { fetchSource } from "../actions/source/fetch.ts"
import { isAssertEqual } from "../actions/test/assert.ts"
import { findTestTitle } from "../actions/test/find.ts"

const OUTPUT = resolve(import.meta.dirname, "..", "tests", "html")
const REPOSITORY = {
  url: "https://github.com/syntax-tree/hast-util-to-html",
  commit: "d115f5beab200c819699e1db9d7eb7b5b1009a21",
}
// Input trees are built by calling the pinned hastscript on statically evaluated arguments
const BUILDERS = { h, s }

type Group = {
  description: string
  source: { repository: string; commit: string; path: string; license: string }
  pipeline: unknown[]
  tests: object[]
}

const skipped = new Map<string, number>()
importToHtml()

function importToHtml() {
  const root = join(
    fetchSource({ repository: REPOSITORY.url, commit: REPOSITORY.commit }),
    "test",
  )
  const byConstruct = new Map<string, Map<string, Group>>()
  let written = 0
  let drifted = 0
  for (const file of readdirSync(root)
    .filter(name => name.endsWith(".js") && name !== "index.js")
    .sort()) {
    const name = basename(file, ".js")
    // The optional-tag files merge into one construct, a group per file
    const construct = name.startsWith("omission") ? "omission" : name
    for (const found of extractCases(readFileSync(join(root, file), "utf8"))) {
      const reference = toHtml(JSON.parse(JSON.stringify(found.tree)), found.options)
      if (reference !== found.html) drifted += 1
      const optionsKey =
        Object.keys(found.options).length > 0 ? ` ${JSON.stringify(found.options)}` : ""
      const description = `hast-util-to-html ${name}${optionsKey}`
      const groups = byConstruct.get(construct) ?? new Map<string, Group>()
      byConstruct.set(construct, groups)
      const stringify =
        Object.keys(found.options).length > 0
          ? ["html.stringify", found.options]
          : ["html.stringify"]
      const group = groups.get(description) ?? {
        description,
        source: {
          repository: REPOSITORY.url,
          commit: REPOSITORY.commit,
          path: `test/${file}`,
          license: "MIT",
        },
        pipeline: [stringify],
        tests: [],
      }
      const comment =
        reference === found.html
          ? {}
          : { comment: `the pinned hast-util-to-html renders: ${reference}` }
      group.tests.push({
        description: found.title,
        ...comment,
        tree: found.tree,
        expected: { html: found.html },
      })
      groups.set(description, group)
      written += 1
    }
  }
  for (const [construct, groups] of byConstruct) {
    writeGroups({
      path: join(OUTPUT, `${construct}.json`),
      repository: REPOSITORY.url,
      groups: [...groups.values()],
    })
  }
  console.log(
    `to-html: ${written} cases in ${byConstruct.size} files, ${drifted} ahead of the pinned reference`,
  )
  for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)
}

// `assert.deepEqual(toHtml(tree, options?), html)`: upstream's HTML is kept verbatim
function extractCases(source: string) {
  const program = parse(source, { ecmaVersion: "latest", sourceType: "module" })
  const cases: {
    title: string
    tree: object
    options: Record<string, unknown>
    html: string
  }[] = []
  const counts = new Map<string, number>()
  ancestor(program, {
    CallExpression(node, _state, ancestors) {
      if (!isAssertEqual(node.callee)) return
      const [call, expected] = node.arguments
      if (
        call?.type !== "CallExpression" ||
        call.callee.type !== "Identifier" ||
        call.callee.name !== "toHtml"
      )
        return
      const constants = resolveConstants({ program, ancestors })
      const tree = evaluateNode(call.arguments[0], constants, 0, BUILDERS)
      const options = call.arguments[1]
        ? evaluateNode(call.arguments[1], constants, 0, BUILDERS)
        : {}
      const html = evaluateNode(expected, constants, 0, BUILDERS)
      if (!isObject(tree))
        return countSkip(
          "a tree JSON cannot hold (NaN, a toString object) or not statically evaluable",
        )
      if (!isObject(options)) return countSkip("options not statically evaluable")
      if (typeof html !== "string") return countSkip("output not statically evaluable")
      const title = findTestTitle(ancestors, constants) ?? "hast-util-to-html"
      const count = (counts.get(title) ?? 0) + 1
      counts.set(title, count)
      cases.push({
        title: count === 1 ? title : `${title} (${count})`,
        tree,
        options: { ...options },
        html,
      })
    },
  })
  return cases
}

function countSkip(reason: string) {
  skipped.set(reason, (skipped.get(reason) ?? 0) + 1)
  return undefined
}
