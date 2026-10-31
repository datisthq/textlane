import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { parse } from "acorn"
import { simple } from "acorn-walk"
import rehypeStringify from "rehype-stringify"
import remarkBreaks from "remark-breaks"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import { unified } from "unified"
import { writeGroups } from "../actions/group/write.ts"
import { evaluateNode, isObject } from "../actions/node/evaluate.ts"
import { fetchSource } from "../actions/source/fetch.ts"

const OUTPUT = join(resolve(import.meta.dirname, "..", "tests", "md"), "breaks.json")
const REMARK = {
  url: "https://github.com/remarkjs/remark-breaks",
  commit: "71f4ff3ca9f08b685dccf2fdb9882d8bbaea3388",
}
const UTIL = {
  url: "https://github.com/syntax-tree/mdast-util-newline-to-break",
  commit: "15c71344dae935541f4c2cf617ed18591a2dbd9d",
}
const html = unified()
  .use(remarkParse)
  .use(remarkBreaks)
  .use(remarkRehype)
  .use(rehypeStringify)
const hast = unified().use(remarkParse).use(remarkBreaks).use(remarkRehype)

// `md.toHtml({ breaks })` stands in for remark-breaks before remark-rehype: remark-breaks'
// fixtures as HTML, upstream's expectation kept verbatim; the util's fixtures as hast trees
let written = 0
const remarkTests = readFixtures(REMARK).map(fixture => {
  const output = String(html.processSync(fixture.in))
  written += 1
  return {
    description: fixture.name,
    ...(output === fixture.out
      ? {}
      : { comment: `remark-breaks expects: ${fixture.out}` }),
    input: fixture.in,
    expected: { html: output },
  }
})
const utilTests = readFixtures(UTIL).map(fixture => {
  const tree = hast.runSync(hast.parse(fixture.in))
  written += 1
  return {
    description: fixture.name,
    input: fixture.in,
    expected: { tree: JSON.parse(JSON.stringify(tree)) },
  }
})
writeGroups({
  path: OUTPUT,
  repository: REMARK.url,
  groups: [
    {
      description: "remark-breaks fixtures",
      source: {
        repository: REMARK.url,
        commit: REMARK.commit,
        path: "test.js",
        license: "MIT",
      },
      pipeline: [["md.parse"], ["md.toHtml", { breaks: true }], ["html.stringify"]],
      tests: remarkTests,
    },
  ],
})
writeGroups({
  path: OUTPUT,
  repository: UTIL.url,
  groups: [
    {
      description: "mdast-util-newline-to-break fixtures, as hast",
      source: {
        repository: UTIL.url,
        commit: UTIL.commit,
        path: "test.js",
        license: "MIT",
      },
      pipeline: [["md.parse"], ["md.toHtml", { breaks: true }]],
      tests: utilTests,
    },
  ],
})
console.log(`breaks: ${written} cases`)

// Both test files list their cases as `const fixtures = [{in, out, name}]`
function readFixtures(repository: { url: string; commit: string }) {
  const root = fetchSource({ repository: repository.url, commit: repository.commit })
  const program = parse(readFileSync(join(root, "test.js"), "utf8"), {
    ecmaVersion: "latest",
    sourceType: "module",
  })
  let fixtures: unknown
  simple(program, {
    VariableDeclarator(node) {
      if (node.id.type === "Identifier" && node.id.name === "fixtures" && node.init)
        fixtures = evaluateNode(node.init, new Map())
    },
  })
  if (!Array.isArray(fixtures)) throw new Error(`no fixtures in ${repository.url}`)
  const read = (fixture: object, key: string) =>
    String(Object.entries(fixture).find(([name]) => name === key)?.[1] ?? "")
  return fixtures.filter(isObject).map(fixture => ({
    in: read(fixture, "in"),
    out: read(fixture, "out"),
    name: read(fixture, "name"),
  }))
}
