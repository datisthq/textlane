import { existsSync, readdirSync, readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { fromHtml } from "hast-util-from-html"
import { toMdast } from "hast-util-to-mdast"
import rehypeFormat from "rehype-format"
import rehypeParse from "rehype-parse"
import rehypeStringify from "rehype-stringify"
import { unified } from "unified"
import { writeGroups } from "../actions/group/write.ts"
import { stringifyTree } from "../actions/markdown/render.ts"
import { fetchSource } from "../actions/source/fetch.ts"

const OUTPUT = resolve(import.meta.dirname, "..", "tests", "html")
const TO_MDAST = {
  url: "https://github.com/syntax-tree/hast-util-to-mdast",
  commit: "caab47b64da47d38792e43f4f4c18b439eeb5233",
}
const FORMAT = {
  url: "https://github.com/rehypejs/rehype-format",
  commit: "0d505500b2cf862ffa42af44ecd7527151b52a6c",
}
// hast-util-to-mdast's harness flags: which assertion a fixture skips, and a parse mode it ignores
const HARNESS_FLAGS = new Set(["fragment", "stringify", "tree"])
// The serializer settings its harness renders Markdown with
const STRINGIFY = { fences: true }

let written = 0
importToMarkdown()
importFormat()
console.log(`html convert: ${written} cases`)

// Each fixture: index.html (`·` stands for a space), options in index.json, Markdown in index.md;
// the harness parses a whole document whatever `fragment` says
function importToMarkdown() {
  const root = join(
    fetchSource({ repository: TO_MDAST.url, commit: TO_MDAST.commit }),
    "test",
    "fixtures",
  )
  const md = new Map<string, object[]>()
  const trees = new Map<string, object[]>()
  for (const name of readdirSync(root).sort()) {
    const configPath = join(root, name, "index.json")
    const config: Record<string, unknown> = existsSync(configPath)
      ? JSON.parse(readFileSync(configPath, "utf8"))
      : {}
    const options = Object.fromEntries(
      Object.entries(config).filter(([key]) => !HARNESS_FLAGS.has(key)),
    )
    const input = readFileSync(join(root, name, "index.html"), "utf8").replace(/·/g, " ")
    const mdast = toMdast(fromHtml(input), options)
    const key = JSON.stringify(options)
    if (config.stringify !== false) {
      const output = stringifyTree({ tree: mdast, gfm: {}, stringify: STRINGIFY })
      const snapshot = readFileSync(join(root, name, "index.md"), "utf8")
      const drift =
        output === snapshot
          ? {}
          : { comment: `hast-util-to-mdast's snapshot: ${snapshot}` }
      md.set(key, [
        ...(md.get(key) ?? []),
        { description: `fixture ${name}`, ...drift, input, expected: { md: output } },
      ])
      written += 1
    }
    if (config.tree !== false) {
      const tree = JSON.parse(
        JSON.stringify(mdast, (field, value) =>
          field === "position" ? undefined : value,
        ),
      )
      trees.set(key, [
        ...(trees.get(key) ?? []),
        { description: `fixture ${name}`, input, expected: { tree } },
      ])
      written += 1
    }
  }
  const source = {
    repository: TO_MDAST.url,
    commit: TO_MDAST.commit,
    path: "test/fixtures",
    license: "MIT",
  }
  const plugin = (key: string) =>
    key === "{}" ? ["html.toMarkdown"] : ["html.toMarkdown", JSON.parse(key)]
  const suffix = (key: string) => (key === "{}" ? "" : ` ${key}`)
  writeGroups({
    path: join(OUTPUT, "to-markdown.json"),
    repository: TO_MDAST.url,
    groups: [
      ...[...md].map(([key, tests]) => ({
        description: `hast-util-to-mdast fixtures${suffix(key)}`,
        source,
        pipeline: [["html.parse"], plugin(key), ["md.gfm"], ["md.stringify", STRINGIFY]],
        tests,
      })),
      ...[...trees].map(([key, tests]) => ({
        description: `hast-util-to-mdast fixtures (tree)${suffix(key)}`,
        source,
        pipeline: [["html.parse"], plugin(key)],
        positions: false,
        tests,
      })),
    ],
  })
}

// Each fixture: input.html, output.html and options in config.json, parsed as a document
function importFormat() {
  const root = join(
    fetchSource({ repository: FORMAT.url, commit: FORMAT.commit }),
    "test",
    "fixtures",
  )
  const groups = new Map<string, object[]>()
  for (const name of readdirSync(root).sort()) {
    const configPath = join(root, name, "config.json")
    const config: Record<string, unknown> = existsSync(configPath)
      ? JSON.parse(readFileSync(configPath, "utf8"))
      : {}
    const input = readFileSync(join(root, name, "input.html"), "utf8")
    const html = String(
      unified()
        .use(rehypeParse)
        .use(rehypeFormat, config)
        .use(rehypeStringify)
        .processSync(input),
    )
    const snapshot = readFileSync(join(root, name, "output.html"), "utf8")
    const drift =
      html === snapshot ? {} : { comment: `rehype-format's snapshot: ${snapshot}` }
    const key = JSON.stringify(config)
    groups.set(key, [
      ...(groups.get(key) ?? []),
      { description: `fixture ${name}`, ...drift, input, expected: { html } },
    ])
    written += 1
  }
  const source = {
    repository: FORMAT.url,
    commit: FORMAT.commit,
    path: "test/fixtures",
    license: "MIT",
  }
  writeGroups({
    path: join(OUTPUT, "format.json"),
    repository: FORMAT.url,
    groups: [...groups].map(([key, tests]) => ({
      description: `rehype-format fixtures${key === "{}" ? "" : ` ${key}`}`,
      source,
      pipeline: [
        ["html.parse"],
        key === "{}" ? ["html.format"] : ["html.format", JSON.parse(key)],
        ["html.stringify"],
      ],
      tests,
    })),
  })
}
