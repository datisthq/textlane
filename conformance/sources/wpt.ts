import { readdirSync, readFileSync } from "node:fs"
import { basename, join, resolve } from "node:path"
import { writeGroups } from "../actions/group/write.ts"
import { parseHtml } from "../actions/markdown/render.ts"
import { fetchSource } from "../actions/source/fetch.ts"

const OUTPUT = resolve(import.meta.dirname, "..", "tests", "html")
const WPT = {
  url: "https://github.com/web-platform-tests/wpt",
  commit: "07ce6e987a1606aec4dc84ef80062d9797330e0b",
}
const RESOURCES = "html/syntax/parsing/resources"

const skipped = new Map<string, number>()
importWpt()

// Tree construction: each `.dat` case's input parsed by the pinned rehype-parse (parse5, which
// passes these suites); trees are stored without positions, which WPT does not test
function importWpt() {
  const root = join(
    fetchSource({
      repository: WPT.url,
      commit: WPT.commit,
      paths: [`/${RESOURCES}/*.dat`],
    }),
    RESOURCES,
  )
  let written = 0
  for (const file of readdirSync(root)
    .filter(name => name.endsWith(".dat"))
    .sort()) {
    const tests: object[] = []
    readCases(readFileSync(join(root, file), "utf8")).forEach((found, index) => {
      if (found.sections.has("document-fragment"))
        return countSkip("a fragment parsed in a context element")
      if (found.sections.has("script-off")) return countSkip("parsing with scripting off")
      if (!found.data.isWellFormed())
        return countSkip("a lone surrogate, which UTF-8 input cannot hold")
      // A few inputs nest deeper than hast-util-from-parse5 recurses; those cannot get a reference tree
      let tree: unknown
      try {
        tree = stripPositions(parseHtml(found.data))
      } catch (error) {
        return countSkip(
          `the reference parser fails (${error instanceof Error ? error.name : "error"})`,
        )
      }
      tests.push({
        description: `case ${index + 1}`,
        input: found.data,
        expected: { tree },
      })
      written += 1
    })
    if (tests.length === 0) continue
    writeGroups({
      path: join(OUTPUT, `wpt-${basename(file, ".dat")}.json`),
      repository: WPT.url,
      groups: [
        {
          description: `WPT ${file}`,
          source: {
            repository: WPT.url,
            commit: WPT.commit,
            path: `${RESOURCES}/${file}`,
            license: "BSD-3-Clause",
          },
          pipeline: [["html.parse"]],
          positions: false,
          tests,
        },
      ],
    })
  }
  console.log(`wpt: ${written} cases`)
  for (const [reason, count] of skipped) console.log(`  skipped ${count}: ${reason}`)
}

// The `.dat` format: `#data`, `#errors`, optional `#document-fragment` and `#script-*`, then
// `#document`, each header a line of its own; `#data` holds the input without its last newline
function readCases(text: string) {
  return text.split(/\n(?=#data\n)/).map(block => {
    const sections = new Map<string, string>()
    let current = ""
    const lines: string[] = []
    const flush = () => {
      if (current) sections.set(current, lines.join("\n"))
      lines.length = 0
    }
    for (const line of block.replace(/\n+$/, "").split("\n")) {
      const header = /^#([a-z-]+)$/.exec(line)
      if (
        header?.[1] &&
        [
          "data",
          "errors",
          "new-errors",
          "document-fragment",
          "script-on",
          "script-off",
          "document",
        ].includes(header[1])
      ) {
        flush()
        current = header[1]
      } else {
        lines.push(line)
      }
    }
    flush()
    return { data: sections.get("data") ?? "", sections }
  })
}

function stripPositions(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(stripPositions)
  if (typeof value !== "object" || value === null) return value
  return Object.fromEntries(
    Object.entries(value)
      .filter(([key]) => key !== "position")
      .map(([key, item]) => [key, stripPositions(item)]),
  )
}

function countSkip(reason: string) {
  skipped.set(reason, (skipped.get(reason) ?? 0) + 1)
  return undefined
}
