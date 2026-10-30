import { existsSync, readdirSync, readFileSync } from "node:fs"
import { createRequire } from "node:module"
import { join, resolve } from "node:path"
import rehypeStringify from "rehype-stringify"
import remarkGfm from "remark-gfm"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import remarkStringify from "remark-stringify"
import remarkToc from "remark-toc"
import { unified } from "unified"
import { writeGroups } from "../actions/group/write.ts"
import { fetchSource } from "../actions/source/fetch.ts"

// remark-heading-id 1.0.1 is CommonJS
const remarkHeadingId = createRequire(import.meta.url)("remark-heading-id")

const TESTS = resolve(import.meta.dirname, "..", "tests")
const PATH = join(TESTS, "md", "toc.json")
const REMARK_TOC = {
  url: "https://github.com/remarkjs/remark-toc",
  commit: "9d0e2764ce3b5a8276e3cdee36c56ff6eecf7477",
  version: "9.0.0",
}
const MDAST_UTIL_TOC = {
  url: "https://github.com/syntax-tree/mdast-util-toc",
  commit: "9fd2abaafa62e3152123e3d4b069fd601844e4a7",
  version: "7.1.0",
}

type Options = Record<string, unknown>
type Source = { repository: string; [key: string]: unknown }
type Render = { gfm?: boolean; headingId?: boolean; html?: boolean }
let written = 0
let skipped = 0

writeGroups({ path: PATH, repository: REMARK_TOC.url, groups: await importRemarkToc() })
writeGroups({
  path: PATH,
  repository: MDAST_UTIL_TOC.url,
  groups: await importMdastUtilToc(),
})
console.log(`toc: ${written} cases, ${skipped} skipped`)

async function importRemarkToc() {
  const root = fetchSource({ repository: REMARK_TOC.url, commit: REMARK_TOC.commit })
  const source = {
    repository: REMARK_TOC.url,
    commit: REMARK_TOC.commit,
    path: "test/index.js",
    license: "MIT",
  }
  const fixtures = readFixtures(join(root, "test", "fixtures"))
  const groups = []
  for (const { name, input, config, output } of fixtures) {
    const expected = await render(input, config, {})
    if (output !== undefined && output !== expected)
      throw new Error(`remark-toc's ${name} fixture disagrees with the pinned reference`)
    groups.push(group(`fixture ${name}`, source, config, {}, [{ name, input, expected }]))
  }
  const own = {
    repository: REMARK_TOC.url,
    version: REMARK_TOC.version,
    path: "lib/index.js",
    license: "MIT",
  }
  groups.push(...(await ownGroups(own)))
  return groups
}

async function importMdastUtilToc() {
  const root = fetchSource({
    repository: MDAST_UTIL_TOC.url,
    commit: MDAST_UTIL_TOC.commit,
  })
  const source = {
    repository: MDAST_UTIL_TOC.url,
    commit: MDAST_UTIL_TOC.commit,
    path: "test/index.js",
    license: "MIT",
  }
  const groups = []
  for (const { name, input, config } of readFixtures(join(root, "test", "fixtures"))) {
    // A test hook sets `data` on every heading; md.headingId's cases cover stored ids
    if ("useCustomHProperty" in config) {
      skipped += 1
      continue
    }
    const expected = await render(input, config, { gfm: true })
    groups.push(
      group(`fixture ${name} through remark-toc`, source, config, { gfm: true }, [
        { name, input, expected },
      ]),
    )
  }
  return groups
}

// Inputs of textlane's own, rendered by the pinned packages
async function ownGroups(source: Source) {
  const document = (body: string) => `# Title\n\n## Contents\n\nOld text\n\n${body}`
  const markdown: [string, string, Options][] = [
    [
      "headings at every depth",
      document("## A\n\n### B\n\n#### C\n\n##### D\n\n###### E\n\n## F"),
      {},
    ],
    ["duplicate headings", document("## Same\n\n## Same\n\n### Same\n\n## Same-1"), {}],
    [
      "a heading named Table of Contents",
      "# Doc\n\n## Table of Contents\n\n## One\n\n## Two",
      {},
    ],
    ["toc in capitals", "## TOC\n\n## One", {}],
    ["no matching heading", "# Doc\n\n## One\n\n## Two", {}],
    ["a matching heading with no headings after", "# Doc\n\n## Contents\n\ntext", {}],
    [
      "only deeper headings before the end",
      "# Doc\n\n## Contents\n\n### Deeper\n\ntext",
      {},
    ],
    [
      "closing heading shallower than the opening",
      "## Contents\n\n### A\n\n# B\n\n## C",
      {},
    ],
    [
      "inline formatting",
      document(
        "## *Emphasis* and **strong**\n\n## `code` here\n\n## A [link](https://example.com) inside\n\n## An ![image](a.png) too",
      ),
      {},
    ],
    ["a link reference", document("## See [ref]\n\n[ref]: https://example.com"), {}],
    ["html in a heading", document("## A <b>b</b> c"), {}],
    [
      "punctuation and unicode",
      document("## Hello, World!\n\n## Ünïcödé — test\n\n## 日本語\n\n## emoji 🎉 here"),
      {},
    ],
    ["empty headings", document("##\n\n## A\n\n##"), {}],
    ["setext headings", "Contents\n--------\n\nA\n===\n\nB\n-", {}],
    ["a heading inside a block quote", document("## A\n\n> ## Quoted\n\n## B"), {}],
    ["headings inside a list", document("## A\n\n- ## Listed\n\n## B"), {}],
    [
      "a custom heading",
      "# Doc\n\n## Overview\n\n## One\n\n## Two",
      { heading: "overview|summary" },
    ],
    ["a custom heading closing a group", "# Doc\n\n## Foo\n\n## One", { heading: "fo+" }],
    ["an empty heading option takes the default", document("## A"), { heading: "" }],
    [
      "skip",
      document("## Example\n\n## Notes\n\n## Real\n\n### Example two"),
      { skip: "example|notes?" },
    ],
    [
      "skip with classes",
      document("## Step 1\n\n## Step 22\n\n## Other"),
      { skip: "step \\d+" },
    ],
    ["minDepth", document("## A\n\n### B\n\n#### C"), { minDepth: 3 }],
    ["maxDepth", document("## A\n\n### B\n\n#### C"), { maxDepth: 3 }],
    [
      "minDepth and maxDepth",
      document("## A\n\n### B\n\n#### C\n\n##### D"),
      { minDepth: 3, maxDepth: 4 },
    ],
    ["minDepth zero is no limit", document("## A\n\n### B"), { minDepth: 0 }],
    ["tight false", document("## A\n\n### B\n\n## C"), { tight: false }],
    ["tight false and flat", document("## A\n\n## B"), { tight: false }],
    ["ordered", document("## A\n\n### B\n\n## C"), { ordered: true }],
    [
      "ordered and loose",
      document("## A\n\n### B\n\n## C"),
      { ordered: true, tight: false },
    ],
    ["prefix", document("## A\n\n### B"), { prefix: "user-content-" }],
    ["a level skipped", document("## A\n\n#### B\n\n### C"), {}],
    ["shallower after deeper", document("#### A\n\n## B\n\n### C"), {}],
    [
      "parents as a type list",
      document("## A\n\n> ## Quoted\n\n## B"),
      { parents: ["root", "blockquote"] },
    ],
    [
      "parents as an object",
      document("## A\n\n> ## Quoted\n\n## B"),
      { parents: { type: "blockquote" } },
    ],
    [
      "parents excluding the root",
      "## Contents\n\n> ## Quoted\n> ## Also",
      { parents: "blockquote" },
    ],
    [
      "an opening heading in a block quote",
      "a\n\nb\n\n> ## Contents\n> ## A\n\n## B",
      { parents: ["root", "blockquote"] },
    ],
    [
      "a closing heading before the opening's index repeats children",
      "> x\n>\n> y\n>\n> ## Contents\n\n## A\n\ntext",
      { parents: ["root", "blockquote"] },
    ],
    ["parents as an empty list lists nothing", document("## A"), { parents: [] }],
    [
      "parents matching on a field",
      document("## A\n\n- ## Listed\n\n1. ## Ordered"),
      { parents: [{ type: "list", ordered: true }, "listItem", "root"] },
    ],
  ]
  const gfm: [string, string, Options][] = [
    ["a footnote call is dropped", document("## Note[^1]\n\n[^1]: text"), {}],
    ["strikethrough", document("## ~~old~~ new"), {}],
  ]
  const html: [string, string, Options][] = [
    ["nested headings", document("## A\n\n### B\n\n## C"), {}],
    ["loose", document("## A\n\n### B\n\n## C"), { tight: false }],
    [
      "ordered with a prefix",
      document("## A\n\n### B"),
      { ordered: true, prefix: "user-content-" },
    ],
    ["inline formatting", document("## *A* `b` [c](d)"), {}],
  ]
  const ids: [string, string][] = [
    ["stored ids are linked", document("## A {#first}\n\n## B")],
    ["a stored id repeating a slug", document("## A {#b}\n\n## B")],
  ]
  const groups = []
  const cases = async (inputs: [string, string, Options][], options: Render) =>
    Promise.all(
      inputs.map(async ([name, input, config]) => ({
        name,
        input,
        config,
        expected: await render(input, config, options),
      })),
    )
  for (const item of await cases(markdown, {}))
    groups.push(group(item.name, source, item.config, {}, [item]))
  for (const item of await cases(gfm, { gfm: true }))
    groups.push(group(item.name, source, item.config, { gfm: true }, [item]))
  for (const item of await cases(html, { html: true }))
    groups.push(
      group(`${item.name} to HTML`, source, item.config, { html: true }, [item]),
    )
  const idCases = await cases(
    ids.map(([name, input]) => [name, input, {}]),
    { headingId: true },
  )
  groups.push(group("ids from md.headingId", source, {}, { headingId: true }, idCases))
  return groups
}

function readFixtures(base: string) {
  return readdirSync(base)
    .filter(name => !name.startsWith("."))
    .sort()
    .map(name => {
      const folder = join(base, name)
      const read = (file: string) =>
        existsSync(join(folder, file))
          ? readFileSync(join(folder, file), "utf8")
          : undefined
      const config: Options = JSON.parse(read("config.json") ?? "{}")
      return { name, input: read("input.md") ?? "", config, output: read("output.md") }
    })
}

async function render(input: string, config: Options, options: Render) {
  const processor = unified().use(remarkParse)
  if (options.gfm) processor.use(remarkGfm)
  if (options.headingId) processor.use(remarkHeadingId)
  processor.use(remarkToc, config)
  if (options.html) processor.use(remarkRehype).use(rehypeStringify)
  else processor.use(remarkStringify)
  return String(await processor.process(input))
}

function group(
  description: string,
  source: Source,
  config: Options,
  options: Render,
  tests: { name: string; input: string; expected: string }[],
) {
  written += tests.length
  const pipeline: unknown[] = [["md.parse"]]
  if (options.gfm) pipeline.push(["md.gfm"])
  if (options.headingId) pipeline.push(["md.headingId"])
  pipeline.push(Object.keys(config).length > 0 ? ["md.toc", config] : ["md.toc"])
  if (options.html) pipeline.push(["md.toHtml"], ["html.stringify"])
  else pipeline.push(["md.stringify"])
  const key = options.html ? "html" : "md"
  return {
    description,
    source,
    pipeline,
    tests: tests.map(test => ({
      description: test.name,
      input: test.input,
      expected: { [key]: test.expected },
    })),
  }
}
