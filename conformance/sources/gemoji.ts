import { readdirSync, readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { nameToEmoji } from "gemoji"
import remarkGemoji from "remark-gemoji"
import remarkParse from "remark-parse"
import remarkStringify from "remark-stringify"
import { unified } from "unified"
import { writeGroups } from "../actions/group/write.ts"
import { fetchSource } from "../actions/source/fetch.ts"

const OUTPUT = join(resolve(import.meta.dirname, "..", "tests", "md"), "gemoji.json")
const REMARK = {
  url: "https://github.com/remarkjs/remark-gemoji",
  commit: "3c2d6b66eda69a22e57234067de71c67fd73e47f",
}
const PIPELINE = [["md.parse"], ["md.gemoji"], ["md.stringify"]]
const reference = unified().use(remarkParse).use(remarkGemoji).use(remarkStringify)

importGemoji()

// remark-gemoji's test checks every name in gemoji's table; the table is the plugin's behaviour,
// so every entry is a case rather than a sample
function importGemoji() {
  const generated = Object.entries(nameToEmoji).map(([name, emoji]) => {
    const input = `Lorem :${name}: ipsum.`
    const md = String(reference.processSync(input))
    // upstream's own expectation: the emoji, escaped where it is Markdown syntax
    const expected = `Lorem ${name === "asterisk" ? "\\" : ""}${emoji} ipsum.\n`
    if (md !== expected)
      console.log(
        `  :${name}: renders ${JSON.stringify(md)}, upstream expects ${JSON.stringify(expected)}`,
      )
    return { description: `:${name}:`, input, expected: { md } }
  })
  const fixtures = join(
    fetchSource({ repository: REMARK.url, commit: REMARK.commit }),
    "test",
    "fixtures",
  )
  const fixtureTests = readdirSync(fixtures)
    .sort()
    .map(name => {
      const input = readFileSync(join(fixtures, name, "input.md"), "utf8")
      const md = String(reference.processSync(input))
      const snapshot = readFileSync(join(fixtures, name, "output.md"), "utf8")
      return {
        description: `fixture ${name}`,
        ...(md === snapshot ? {} : { comment: `remark-gemoji's snapshot: ${snapshot}` }),
        input,
        expected: { md },
      }
    })
  const source = (path: string) => ({
    repository: REMARK.url,
    commit: REMARK.commit,
    path,
    license: "MIT",
  })
  writeGroups({
    path: OUTPUT,
    repository: REMARK.url,
    groups: [
      {
        description: "remark-gemoji, every gemoji 8.1.0 name",
        comment:
          "Generated from the gemoji package's nameToEmoji, as remark-gemoji's own test is",
        source: source("test/index.js"),
        pipeline: PIPELINE,
        tests: generated,
      },
      {
        description: "remark-gemoji fixtures",
        source: source("test/fixtures"),
        pipeline: PIPELINE,
        tests: fixtureTests,
      },
    ],
  })
  console.log(`gemoji: ${generated.length + fixtureTests.length} cases`)
}
