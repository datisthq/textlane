import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import rehypeStringify from "rehype-stringify"
import remarkCjkFriendly from "remark-cjk-friendly"
import remarkCjkFriendlyGfmStrikethrough from "remark-cjk-friendly-gfm-strikethrough"
import remarkGfm from "remark-gfm"
import remarkParse from "remark-parse"
import remarkRehype from "remark-rehype"
import remarkStringify from "remark-stringify"
import { unified } from "unified"
import { writeGroups } from "../actions/group/write.ts"
import { fetchSource } from "../actions/source/fetch.ts"

const OUTPUT = join(
  resolve(import.meta.dirname, "..", "tests", "md"),
  "cjk-friendly.json",
)
const CJK = {
  url: "https://github.com/tats-u/markdown-cjk-friendly",
  commit: "71d6b25d406a31d0303b102d582bd675067ed73f",
}
// Shared test cases, one per paragraph; the GFM ones add strikethrough
const FILES: [string, boolean][] = [
  ["strong.md", false],
  ["korean.md", false],
  ["non-bmp.md", false],
  ["pseudo-emoji.md", false],
  ["underscore-cjk-punct.md", false],
  ["gfm-strikethrough.md", true],
  ["gfm-non-cjk.md", true],
]
const HTML = { allowDangerousHtml: true }
type Source = { repository: string; commit: string; path: string; license: string }

const root = fetchSource({ repository: CJK.url, commit: CJK.commit })
const source = (path: string): Source => ({
  repository: CJK.url,
  commit: CJK.commit,
  path,
  license: "MIT",
})
const groups = []
let written = 0
for (const [file, gfm] of FILES) {
  const inputs = readFileSync(join(root, "testcases", file), "utf8")
    .split(/\n{2,}/)
    .map(paragraph => paragraph.trim())
    .filter(Boolean)
  groups.push(htmlGroup(`${file}`, source(`testcases/${file}`), inputs, gfm))
  groups.push(
    markdownGroup(`${file} (md.stringify)`, source(`testcases/${file}`), inputs, gfm),
  )
}
// CommonMark's emphasis examples hold with the extension on: CJK-friendly changes only
// text next to CJK
const commonmark: { markdown: string; section: string; example: number }[] = JSON.parse(
  readFileSync(join(root, "testcases", "commonmark.json"), "utf8"),
)
const emphasis = commonmark.filter(
  example => example.section === "Emphasis and strong emphasis",
)
groups.push(
  htmlGroup(
    "commonmark.json, emphasis and strong emphasis",
    source("testcases/commonmark.json"),
    emphasis.map(example => example.markdown),
    false,
  ),
)
writeGroups({ path: OUTPUT, repository: CJK.url, groups })
console.log(`cjk: ${written} cases`)

function htmlGroup(description: string, from: Source, inputs: string[], gfm: boolean) {
  const processor = unified()
    .use(remarkParse)
    .use(
      gfm
        ? [remarkGfm, remarkCjkFriendly, remarkCjkFriendlyGfmStrikethrough]
        : [remarkCjkFriendly],
    )
    .use(remarkRehype, HTML)
    .use(rehypeStringify, HTML)
  written += inputs.length
  return {
    description,
    source: from,
    pipeline: [
      ["md.parse"],
      ...(gfm ? [["md.gfm"]] : []),
      ["md.cjkFriendly"],
      ["md.toHtml", HTML],
      ["html.stringify", HTML],
    ],
    tests: inputs.map(input => ({
      description: JSON.stringify(input),
      input,
      expected: { html: String(processor.processSync(input)) },
    })),
  }
}

// remark-cjk-friendly's serializer escapes attention so it parses back; `md.stringify` does
// not port it yet, so these stay in pending.json
function markdownGroup(
  description: string,
  from: Source,
  inputs: string[],
  gfm: boolean,
) {
  const processor = unified()
    .use(remarkParse)
    .use(
      gfm
        ? [remarkGfm, remarkCjkFriendly, remarkCjkFriendlyGfmStrikethrough]
        : [remarkCjkFriendly],
    )
    .use(remarkStringify)
  written += inputs.length
  return {
    description,
    source: from,
    pipeline: [
      ["md.parse"],
      ...(gfm ? [["md.gfm"]] : []),
      ["md.cjkFriendly"],
      ["md.stringify"],
    ],
    tests: inputs.map(input => ({
      description: JSON.stringify(input),
      input,
      expected: { md: String(processor.processSync(input)) },
    })),
  }
}
